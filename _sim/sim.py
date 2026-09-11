"""A conversation simulator, so one difference between two agents can be moved at a time.

Comparing Suffice against OpenCode by running both binaries makes every difference move at once:
prompt, tool schemas, tool output envelopes, message layout and request parameters all change
together, and a run costs six to twelve requests before it says anything. Four such runs left the
cause of the cost gap unidentified.

This replaces both binaries with the only part that matters - the loop that talks to the model. The
prefix is assembled from the *captured wire bytes* of real runs (`_labs/wire/`), not rebuilt, so an
arm is byte-identical to the agent it imitates except where an option says otherwise. The tools are
implemented once in `tools.py` and rendered through whichever envelope the arm asks for.

Each arm is stopped the moment the model asks for its first `read`, because the number under test -
the window it chooses - is decided there and costs two or three requests to learn instead of twelve.

    python sim.py --arm oc
    python sim.py --ladder            # every arm, one after another

Requires a relay on --port (metering plus the credential):
    python ..\\..\\measurement\\measurement\\relay.py --port 8799 --label sim --out sim.jsonl
"""

import argparse
import io
import json
import os
import shutil
import sys
import time

import httpx

import tools as T

HERE = os.path.dirname(os.path.abspath(__file__))
LABS = os.path.dirname(HERE)
WIRE = os.path.join(LABS, "wire")
SEED = os.path.join(os.path.dirname(LABS), "measurement", "measurement", "seed-rs")
WORK = os.path.join(HERE, "work")

# Paths baked into the captured prefixes, rewritten to this simulator's corpus so the model is not
# told about a directory that does not exist here.
CAPTURED_ROOTS = [
    r"C:\Users\Bedirhan\Desktop\agent\ab\par-opencode",
    r"C:\Users\azsxd\codex\_labs\work\wire-sufficefork-rep2",
    r"C:\Users\azsxd\codex\_labs\work\wire-sufficefork-rep4",
]

PRICE = {"fresh": 0.075e-6, "cached": 0.015e-6, "out": 0.25e-6}


def cap(path):
    return io.open(path, encoding="utf-8", errors="replace").read()


def retarget(text):
    for old in CAPTURED_ROOTS:
        text = text.replace(old, WORK)
        text = text.replace(old.replace("\\", "/"), WORK.replace("\\", "/"))
    return text


def tool_specs(which, read_from=None, read_desc_edit=None, extra_tools=None, drop_tools=None):
    """The captured specs of one agent, optionally with its `read` taken from the other.

    Swapping one spec is how a rung isolates a tool description from the rest of a tool surface:
    rung 3 moved twelve specs at once and the window went from 60 lines to unset.
    """
    d = os.path.join(WIRE, which, "tools")
    out = []
    for name in sorted(os.listdir(d)):
        if not name.endswith(".json"):
            continue
        spec = json.loads(retarget(cap(os.path.join(d, name))))
        if name == "read.json" and read_from:
            spec = json.loads(retarget(cap(os.path.join(WIRE, read_from, "tools", "read.json"))))
        out.append(spec)
    for extra in extra_tools or []:
        other = "opencode" if which != "opencode" else "suf"
        out.append(json.loads(retarget(cap(os.path.join(WIRE, other, "tools", extra + ".json")))))
    if drop_tools:
        out = [s for s in out if (s.get("function", s)).get("name") not in drop_tools]
    if read_desc_edit:
        for spec in out:
            fn = spec.get("function", spec)
            if fn.get("name") == "read":
                before, after = read_desc_edit
                if before not in fn["description"]:
                    raise SystemExit(f"read_desc_edit did not match: {before[:60]!r}")
                fn["description"] = fn["description"].replace(before, after)
    return out


# --------------------------------------------------------------------------- arms
#
# The ladder runs from a faithful OpenCode down to a faithful Suffice, moving one thing per rung.
# `base` names the rung it is copied from, so each entry lists only its own change.

ARMS = {
    "0-oc": dict(
        prefix="opencode",      # one system message: prompt + model id + <env> + AGENTS.md + skills
        toolset="opencode",     # bash/edit/write/task/todowrite/webfetch/skill + glob/grep/read
        read_envelope="oc",     # <path>/<type>/<content>, always closes with the total line count
        max_tokens=32000,
        parallel_tool_calls=None,
    ),
    "1-oc+sufprompt": dict(
        prefix="opencode", toolset="opencode", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        swap_system="suf",   # suffice's instructions_template in OpenCode's slot
    ),
    "2-oc+suflayout": dict(
        prefix="suf",   # system + system(skills/permissions) + user(AGENTS.md) + user(task)
        toolset="opencode", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    "3-oc+suftools": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    # Rung 3 moved twelve specs together. These two bracket it: one keeps OpenCode's whole tool
    # surface and takes only `read` from Suffice, the other does the reverse.
    "3a-octools+sufreadspec": dict(
        prefix="suf", toolset="opencode", read_from="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    "3b-suftools+ocreadspec": dict(
        prefix="suf", toolset="suf", read_from="opencode", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    # And this one removes a single sentence from Suffice's `read`, the only substantive line
    # OpenCode's does not have.
    "3c-suftools-minus-byte-sentence": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        read_desc_edit=(
            ", and at most about 32000 bytes: `limit` counts lines but the ceiling is bytes, "
            "so a file of long lines stops earlier than the line count suggests",
            "",
        ),
    ),
    # Neither read spec explains rung 3, so the discipline lives in the rest of the surface. These
    # bracket the two tools OpenCode has for spending less context - `task`, which its own prompt
    # calls "in order to reduce context usage", and `todowrite`.
    "3d-suftools+octask+octodo": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        extra_tools=["task", "todowrite"],
    ),
    "3e-octools-minus-task-todo": dict(
        prefix="suf", toolset="opencode", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        drop_tools={"task", "todowrite"},
    ),
    "3b2-suftools+ocreadspec-again": dict(
        prefix="suf", toolset="suf", read_from="opencode", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    # Still unexplained after read/task/todowrite. What is left is `bash` against `exec_command`
    # - OpenCode's single largest spec at 5,969 B - and the goal tools, whose text says "token
    # budget" and "remaining token budget" four times between them.
    "3f-suftools-bash-for-exec": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        extra_tools=["bash"], drop_tools={"exec_command"},
    ),
    "3g-suftools-minus-goals": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        drop_tools={"create_goal", "get_goal", "update_goal", "update_plan"},
    ),
    # Each single removal moved 2000 -> 120-200 and none reached OpenCode's 80, so this stacks all
    # three at once: OpenCode's `bash` in place of `exec_command`, no goal or plan tools, and the
    # byte sentence gone from `read`.
    "3h-suftools-all-three": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        extra_tools=["bash"],
        drop_tools={"exec_command", "create_goal", "get_goal", "update_goal", "update_plan"},
        read_desc_edit=(
            ", and at most about 32000 bytes: `limit` counts lines but the ceiling is bytes, "
            "so a file of long lines stops earlier than the line count suggests",
            "",
        ),
    ),
    # 3g dropped four specs. `update_plan` is innocuous; the three goal tools are the ones whose
    # text says "token_budget", "remaining token budget", "budgeted goal" and "budget is nearly
    # exhausted". This drops only those.
    "3i-suftools-minus-goals-only": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
        drop_tools={"create_goal", "get_goal", "update_goal"},
    ),
    # Repeats, to say what part of the spread above is just run-to-run variance.
    "0r-oc-again": dict(
        prefix="opencode", toolset="opencode", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    "3r-suftools-again": dict(
        prefix="suf", toolset="suf", read_envelope="oc",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    "4-oc+sufread": dict(
        prefix="suf", toolset="suf", read_envelope="suffice",
        max_tokens=32000, parallel_tool_calls=None,
    ),
    "5-suf": dict(
        prefix="suf", toolset="suf", read_envelope="suffice",
        max_tokens=None, parallel_tool_calls=True,
    ),
}


def build_messages(arm):
    src = arm["prefix"]
    if src == "opencode":
        system = retarget(cap(os.path.join(WIRE, "opencode", "msg00_system.txt")))
        if arm.get("swap_system"):
            # Replace only the agent's own prompt, keeping the model id, <env>, AGENTS.md and the
            # skills tail that follow it - the same surgery the `wire-suffice` arm did for real.
            marker = "You are powered by the model named"
            tail = system[system.index(marker):]
            other = retarget(cap(os.path.join(WIRE, arm["swap_system"], "msg00_system.txt")))
            system = other.rstrip() + "\n\n" + tail
        user = retarget(cap(os.path.join(WIRE, "opencode", "msg01_user.txt")))
        return [{"role": "system", "content": system}, {"role": "user", "content": user}]

    d = os.path.join(WIRE, "suf")
    return [
        {"role": "system", "content": retarget(cap(os.path.join(d, "msg00_system.txt")))},
        {"role": "system", "content": retarget(cap(os.path.join(d, "msg01_system.txt")))},
        {"role": "user", "content": retarget(cap(os.path.join(d, "msg02_user.txt")))},
        {"role": "user", "content": retarget(cap(os.path.join(WIRE, "opencode", "msg01_user.txt")))},
    ]


# --------------------------------------------------------------------------- tool dispatch

READ_NAMES = {"read"}
NOOP_OK = {
    "todowrite": "Todos updated",
    "update_plan": "Plan updated",
    "skill": "No skill loaded",
    "view_image": "Image attached",
    "get_goal": "No goal set",
    "create_goal": "Goal created",
    "update_goal": "Goal updated",
    "write_stdin": "(no output)",
    "request_user_input": "The user did not respond; continue with your best judgement.",
    "webfetch": "Fetch disabled in this environment",
    "task": "Subagent disabled in this environment; do the work yourself.",
}


def run_tool(name, args, arm):
    if name == "glob":
        return T.glob_tool(WORK, args.get("pattern", ""), args.get("path"))
    if name == "grep":
        return T.grep_tool(WORK, args.get("pattern", ""), args.get("path"), args.get("include"))
    if name == "read":
        if "paths" in args:
            return T.read_suffice_batch(WORK, args.get("paths"), args.get("offset"), args.get("limit"))
        return T.read_tool(WORK, args.get("filePath", ""), args.get("offset"),
                           args.get("limit"), envelope=arm["read_envelope"])
    if name in ("bash", "exec_command"):
        return T.shell_tool(WORK, args.get("command") or args.get("cmd") or "")
    if name in ("write", "edit", "apply_patch"):
        return "Success. (writes are not applied in the simulator)"
    return NOOP_OK.get(name, f"{name}: not available in this environment")


# --------------------------------------------------------------------------- the loop


def chat(client, url, body):
    """One streaming request, accumulated into a message plus its usage."""
    content, calls, usage = "", {}, None
    with client.stream("POST", url, json=body) as resp:
        if resp.status_code != 200:
            raise RuntimeError(f"HTTP {resp.status_code}: {resp.read()[:400]!r}")
        for line in resp.iter_lines():
            line = line.strip()
            if not line.startswith("data:"):
                continue
            payload = line[5:].strip()
            if payload == "[DONE]":
                continue
            try:
                obj = json.loads(payload)
            except ValueError:
                continue
            if obj.get("usage"):
                usage = obj["usage"]
            for choice in obj.get("choices") or []:
                delta = choice.get("delta") or {}
                content += delta.get("content") or ""
                for tc in delta.get("tool_calls") or []:
                    slot = calls.setdefault(tc.get("index", 0), {"id": "", "name": "", "args": ""})
                    if tc.get("id"):
                        slot["id"] = tc["id"]
                    fn = tc.get("function") or {}
                    if fn.get("name"):
                        slot["name"] = fn["name"]
                    slot["args"] += fn.get("arguments") or ""
    return content, [calls[k] for k in sorted(calls)], usage


def run_arm(name, arm, url, steps, stop_on_read, verbose):
    messages = build_messages(arm)
    specs = tool_specs(
        arm["toolset"], arm.get("read_from"), arm.get("read_desc_edit"),
        arm.get("extra_tools"), arm.get("drop_tools"),
    )
    client = httpx.Client(timeout=httpx.Timeout(600.0, connect=30.0))

    fresh = cached = out_tok = 0
    read_limits, trace = [], []

    for step in range(1, steps + 1):
        body = {
            "model": "glm-5.3-flash",
            "messages": messages,
            "tools": specs,
            "tool_choice": "auto",
            "stream": True,
            "stream_options": {"include_usage": True},
            "thinking": {"type": "enabled"},
            "reasoning_effort": "high",
        }
        if arm.get("max_tokens"):
            body["max_tokens"] = arm["max_tokens"]
        if arm.get("parallel_tool_calls") is not None:
            body["parallel_tool_calls"] = arm["parallel_tool_calls"]

        content, calls, usage = chat(client, url, body)
        if usage:
            pt = usage.get("prompt_tokens", 0)
            ct = (usage.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
            fresh += pt - ct
            cached += ct
            out_tok += usage.get("completion_tokens", 0)

        names = [c["name"] for c in calls]
        trace.append({"step": step, "calls": names, "text": content[:200]})
        if verbose:
            print(f"  [{step}] {names or 'no tool calls'}")

        if not calls:
            break

        messages.append({
            "role": "assistant",
            "content": content or None,
            "tool_calls": [
                {"id": c["id"], "type": "function",
                 "function": {"name": c["name"], "arguments": c["args"]}}
                for c in calls
            ],
        })

        hit_read = False
        for c in calls:
            try:
                args = json.loads(c["args"] or "{}")
            except ValueError:
                args = {}
            if c["name"] in READ_NAMES:
                hit_read = True
                if "paths" in args:
                    for item in args.get("paths") or []:
                        lim = item.get("limit", args.get("limit")) if isinstance(item, dict) else args.get("limit")
                        read_limits.append(lim)
                else:
                    read_limits.append(args.get("limit"))
            result = run_tool(c["name"], args, arm)
            messages.append({"role": "tool", "tool_call_id": c["id"], "content": result})

        if hit_read and stop_on_read:
            break

    cost = fresh * PRICE["fresh"] + cached * PRICE["cached"] + out_tok * PRICE["out"]
    return {
        "arm": name, "steps": len(trace), "fresh": fresh, "cached": cached,
        "out": out_tok, "cost": round(cost, 5),
        "read_calls": len(read_limits),
        "read_limits": read_limits,
        "trace": trace,
    }


def prepare_corpus():
    if os.path.isdir(WORK):
        shutil.rmtree(WORK, ignore_errors=True)
        time.sleep(0.3)
    shutil.copytree(SEED, WORK)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", action="append", help="arm name; repeatable. Default: the whole ladder")
    ap.add_argument("--port", type=int, default=8799)
    ap.add_argument("--steps", type=int, default=4, help="hard ceiling on requests per arm")
    ap.add_argument("--all-steps", action="store_true", help="do not stop at the first read")
    ap.add_argument("--reps", type=int, default=1, help="repeat each arm; the spread is the finding")
    ap.add_argument("--out", default=os.path.join(HERE, "results.jsonl"))
    args = ap.parse_args()

    url = f"http://127.0.0.1:{args.port}/api/paas/v4/chat/completions"
    names = args.arm or list(ARMS)
    prepare_corpus()

    rows = []
    for name in names:
        if name not in ARMS:
            print(f"unknown arm {name!r}; known: {', '.join(ARMS)}")
            return 2
        for rep in range(1, args.reps + 1):
            label = name if args.reps == 1 else f"{name}#{rep}"
            print(f"=== {label}")
            row = run_arm(name, ARMS[name], url, args.steps, not args.all_steps, verbose=True)
            row["arm"] = label
            rows.append(row)
            lim = [x for x in row["read_limits"] if x]
            med = sorted(lim)[len(lim) // 2] if lim else None
            print(f"    steps {row['steps']}  cost ${row['cost']:.4f}  "
                  f"read calls {row['read_calls']}  limits {row['read_limits'][:12]}  median {med}")
            with io.open(args.out, "a", encoding="utf-8") as fh:
                fh.write(json.dumps(row) + "\n")

    print()
    print(f"{'arm':32} {'reads':>6} {'set':>8} {'median':>7} {'cost':>9}")
    for r in rows:
        lim = [x for x in r["read_limits"] if x]
        med = sorted(lim)[len(lim) // 2] if lim else "-"
        setn = f"{len(lim)}/{r['read_calls']}"
        print(f"{r['arm']:32} {r['read_calls']:>6} {setn:>8} {str(med):>7} ${r['cost']:>8.4f}")
    print(f"total ${sum(r['cost'] for r in rows):.4f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
