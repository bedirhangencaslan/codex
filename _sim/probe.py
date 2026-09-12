"""A hand-drivable rig for one question: what makes this model ask for wide read windows?

Everything before this measured an opening move. `swap.py` replays one request and scores the
`limit` values in the reply; `sim.py` closes the loop but stops at the first `read` by default. Two
real runs showed why that is not enough - serialising the fork's reads took bounded reads from 27/46
to 46/46 and left the median window at 280 against OpenCode's 80. Whether a window is set and how
wide it is are different questions, and only the second costs money.

So: run the read phase out, on the real corpus, with real tools, and report the **bytes that come
back per read**. Not dollars - the budget does not stretch to costing whole tasks, and bytes per read
is the thing dollars are made of anyway.

    python probe.py --base suf --reps 3                      # the fork, as it ships
    python probe.py --base oc --base suf --reps 3            # both, interleaved
    python probe.py --base suf --mut cap:flat --reps 3       # one knob moved

Arms are named `base+mut+mut`. Every batch interleaves its arms round-robin, because running them in
blocks once put a false 29% -> 93% on a variable that turned out to do nothing: something drifts
inside a session, and interleaving spreads it over every arm instead of dumping it on whichever ran
first.

Needs a relay on --port holding the credential:
    python ..\\..\\measurement\\measurement\\relay.py --port 8799 --label probe --out probe.relay.jsonl
"""

import argparse
import copy
import io
import json
import os
import shutil
import statistics
import sys
import time

import httpx

import tools as T

HERE = os.path.dirname(os.path.abspath(__file__))
LABS = os.path.dirname(HERE)
LOGS = os.path.join(LABS, "logs")
SEED = os.path.join(os.path.dirname(LABS), "measurement", "measurement", "seed-rs")
WORK = os.path.join(HERE, "probework")
SUFFICE_SRC = os.path.join(os.path.dirname(LABS), "suffice", "codex-rs")

PRICE = {"fresh": 0.075e-6, "cached": 0.015e-6, "out": 0.25e-6}

# The bodies these bases are cut from. Both are from the same session on 2026-09-12, which is the
# only pair in the archive taken side by side - and the archive's own rule is that a baseline is a
# run taken beside the run it is compared to.
BASE_BODIES = {
    "suf": os.path.join(LOGS, "wire-sufficefork-rep30", "relay.jsonl.suffice.body.json"),
    "oc": os.path.join(LOGS, "wire-stock-rep30", "relay.jsonl.stock.body.json"),
}

# `read.rs` constants, mirrored so the probe cuts where the handler cuts.
PER_FILE_BUDGET_BYTES = 32_000
LIMIT_BYTES_PER_LINE = 100
NUMBERING_OVERHEAD_DIVISOR = 2
NUMBERING_OVERHEAD_FLOOR_BYTES = 64
NOTE_RESERVE_BYTES = 96
OC_MAX_BYTES = 50 * 1024
MAX_LINE_BYTES = 2_000
DEFAULT_LINE_LIMIT = 2_000


def load(path):
    return json.loads(io.open(path, encoding="utf-8", errors="replace").read())


def retarget(text, work=WORK):
    """Point the captured prefix at this probe's corpus copy, not the run it was captured from."""
    for run in ("wire-sufficefork-rep30", "wire-stock-rep30", "wire-sufficefork-rep22",
                "wire-stock-rep11", "wire-sufficefork-rep32", "wire-stock-rep32"):
        old = os.path.join(LABS, "work", run)
        text = text.replace(old, work).replace(old.replace("\\", "/"), work.replace("\\", "/"))
    return text


# --------------------------------------------------------------------------- the read tool
#
# This is the instrument. Both agents' handlers are reproduced from source rather than from memory,
# and the parts under test are policy flags rather than agent identities, so an arm can carry the
# fork's envelope with OpenCode's cap or any other combination.


def _numbered_demand(size):
    """`read.rs:807-809` - what the body costs once every line carries its number."""
    return size + size // NUMBERING_OVERHEAD_DIVISOR + NUMBERING_OVERHEAD_FLOOR_BYTES


def read_tool(root, file_path, offset, limit, policy):
    fp = file_path if os.path.isabs(file_path) else os.path.join(root, file_path)
    fp = os.path.abspath(fp)
    if not os.path.isfile(fp):
        return f"File not found: {fp}", None

    raw = io.open(fp, encoding="utf-8", errors="replace").read()
    lines = raw.splitlines()
    total = len(lines)
    start = max(1, offset or 1)
    lim = limit if limit is not None else DEFAULT_LINE_LIMIT

    if policy["cap"] == "oc":
        cap = OC_MAX_BYTES
    else:
        # `read.rs:810-815`. The second `.min` is the coupling under test: asking for fewer lines
        # also reserves fewer bytes, so a read that gets cut has a mechanical reason to come back
        # asking for more lines. OpenCode's cap is flat and has no such term.
        cap = min(_numbered_demand(len(raw.encode("utf-8"))), PER_FILE_BUDGET_BYTES)
        if limit is not None and policy["cap"] == "suf":
            cap = min(cap, limit * LIMIT_BYTES_PER_LINE)

    kept, size, hit_cap, clipped = [], 0, False, 0
    for i in range(start - 1, min(total, start - 1 + lim)):
        text = lines[i]
        if len(text.encode("utf-8")) > MAX_LINE_BYTES:
            text = text[:MAX_LINE_BYTES] + " ... (line truncated)"
            clipped += 1
        row = f"{i + 1}: {text}"
        # The fork charges the whole numbered row; OpenCode charges the line text only
        # (`read.ts:163`), which is why its 50 KB cap in practice passes rather more than 50 KB.
        charged = len(row) + 1 if policy["envelope"] == "suf" else len(text.encode("utf-8")) + 1
        if size + charged > cap:
            hit_cap = True
            break
        kept.append(row)
        size += charged

    shown_to = start - 1 + len(kept)
    more = hit_cap or shown_to < total
    meta = {"file": os.path.relpath(fp, root), "offset": offset, "limit": limit,
            "file_lines": total, "kept": len(kept), "hit_cap": hit_cap, "cap": cap}

    if policy["envelope"] == "oc":
        body = f"<path>{fp}</path>\n<type>file</type>\n<content>\n" + "\n".join(kept)
        if hit_cap:
            # OpenCode aborts the file stream when the cap fires, so its line counter is only
            # "scanned so far" and it cannot name a total here even if it wanted to (`read.ts:170`).
            body += f"\n\n(Output capped at 50 KB. Showing lines {start}-{shown_to}. Use offset={shown_to + 1} to continue.)"
        elif more:
            body += f"\n\n(Showing lines {start}-{shown_to} of {total}. Use offset={shown_to + 1} to continue.)"
        else:
            body += f"\n\n(End of file - total {total} lines)"
        return body + "\n</content>", meta

    notes = []
    disclose = policy["disclose"]
    if start > total and total > 0:
        notes.append(f"offset {start} is past the end; {total} lines")
    elif more:
        where = f"showing lines {start}-{shown_to}"
        if disclose == "total":
            where += f" of {total}"
        if hit_cap:
            # `read.rs:432-438` labels this with the constant, not with the cap that actually fired.
            shown_cap = cap if policy["label"] == "honest" else PER_FILE_BUDGET_BYTES
            notes.append(f"output capped at {shown_cap // 1024} KB, {where}; use offset {shown_to + 1} to continue")
        else:
            notes.append(f"{where}; use offset {shown_to + 1} to continue")
    else:
        notes.append(f"end of file, {total} lines" if disclose == "total" else "end of file")
    if clipped:
        notes.append(f"{clipped} long line(s) clipped")

    note = ", ".join(notes)
    if policy["clip_note"]:
        note = note.encode("utf-8")[:NOTE_RESERVE_BYTES].decode("utf-8", "ignore")
    if disclose == "silent" and not more:
        note = ""
    head = f"===== {fp} ({note})\n" if note else f"===== {fp}\n"
    return head + "\n".join(kept) + "\n", meta


DEFAULT_POLICY = {
    "suf": {"cap": "suf", "envelope": "suf", "disclose": "total", "label": "const", "clip_note": True},
    "oc": {"cap": "oc", "envelope": "oc", "disclose": "total", "label": "const", "clip_note": False},
}


# --------------------------------------------------------------------------- bases and mutations


def _line_count_listing():
    """What the fork's own measuring step returned: every file with its exact length."""
    rows = []
    for base, dirs, files in os.walk(os.path.join(WORK, "codex-api", "src")):
        for f in sorted(files):
            if f.endswith(".rs"):
                p = os.path.join(base, f)
                n = len(io.open(p, encoding="utf-8", errors="replace").read().splitlines())
                rows.append(f"{p}\t{n}")
    return "\n".join(rows) + "\n"


# The task is the one input both agents receive identically, so it has never been varied - which
# makes it the least-examined thing in the whole surface. `every` is the word the fork's own
# reasoning keeps returning to: "To 'read every' maybe full reading", "User says read every file;
# we need read whole files". These two say the same job with and without that word.
TASKS = {
    "wire": None,  # filled from the base at import; the benchmark's own text
    "summarize": (
        "Write `WIRE.md` describing the `.rs` files under `codex-api/src/`.\n\n"
        "For each file, write one line: the path, what it translates or handles, and one "
        "`file:line`\ncitation backing it. Group the lines under the directory each file lives in.\n\n"
        "Do not modify any file except `WIRE.md`.\n\n"
        "STOP CONDITION. When `WIRE.md` has a line for each `.rs` file under `codex-api/src/`, "
        "count the\nfiles you covered and print `CHECKPOINT REACHED: <n> files`. Then stop. Do not "
        "review anything\nelse and do not write any other file.\n"
    ),
}


def oc_sections():
    """OpenCode's single system message, cut into the four pieces it is assembled from."""
    s = retarget(load(BASE_BODIES["oc"])["messages"][0]["content"])
    i_env = s.index("Here is some useful information")
    i_agents = s.index("Instructions from: ", i_env)
    i_skills = s.index("Skills provide", i_agents)
    return s[:i_env].rstrip(), s[i_env:i_agents].rstrip(), s[i_agents:i_skills].rstrip(), s[i_skills:].rstrip()


def base_arm(name):
    body = load(BASE_BODIES[name])
    return {
        "messages": [{k: retarget(v) if isinstance(v, str) else v for k, v in m.items()}
                     for m in body["messages"]],
        "tools": json.loads(retarget(json.dumps(body["tools"]))),
        "parallel": body.get("parallel_tool_calls"),
        "policy": dict(DEFAULT_POLICY[name]),
    }


def _drop_section(text, heading):
    """Remove one `# ` or `## ` section: from its heading to the next heading of the same depth."""
    lines = text.splitlines()
    starts = [i for i, l in enumerate(lines) if l.strip() == heading]
    if not starts:
        raise SystemExit(f"section not found: {heading!r}")
    i = starts[0]
    depth = len(heading) - len(heading.lstrip("#"))
    j = i + 1
    while j < len(lines):
        stripped = lines[j].lstrip("#")
        d = len(lines[j]) - len(stripped)
        if lines[j].startswith("#") and d <= depth:
            break
        j += 1
    return "\n".join(lines[:i] + lines[j:])


def apply_mut(arm, spec):
    key, _, val = spec.partition(":")
    msgs = arm["messages"]

    if key == "tools":
        arm["tools"] = base_arm(val)["tools"]
    elif key == "tool":
        # `tool:read=oc` - one spec taken from the other agent, the rest left alone.
        which, _, src = val.partition("=")
        spec = next(t for t in base_arm(src)["tools"] if t.get("function", t)["name"] == which)
        arm["tools"] = [spec if t.get("function", t)["name"] == which else t for t in arm["tools"]]
    elif key == "param":
        # `param:exec_command-max_output_tokens` - one parameter removed, description untouched.
        # Our shell tool carries ten parameters against OpenCode's three, and one of them is a
        # budget knob; `shell_spec.rs:60-62` already records that this model read an earlier wording
        # of it as its *context* budget and rationed its read windows against it.
        tool, _, drop = val.partition("-")
        for t in arm["tools"]:
            fn = t.get("function", t)
            if fn["name"] == tool:
                fn["parameters"]["properties"].pop(drop, None)
                req = fn["parameters"].get("required")
                if req and drop in req:
                    req.remove(drop)
    elif key == "desc":
        # `desc:exec_command=bash` - their prose on our schema. Splits the description from the
        # parameter list, which `swap:` moves together.
        mine, _, theirs = val.partition("=")
        src = next(t for t in base_arm("oc")["tools"] if t.get("function", t)["name"] == theirs)
        for t in arm["tools"]:
            fn = t.get("function", t)
            if fn["name"] == mine:
                fn["description"] = src.get("function", src)["description"]
    elif key == "swap":
        # `swap:exec_command=bash` - ours out, theirs in, for a pair that does the same job under
        # different names. `tool:` cannot express this because it matches on the name.
        mine, _, theirs = val.partition("=")
        spec = next(t for t in base_arm("oc")["tools"] if t.get("function", t)["name"] == theirs)
        arm["tools"] = [t for t in arm["tools"] if t.get("function", t)["name"] != mine] + [spec]
    elif key == "drop":
        names = {"goals": {"get_goal", "create_goal", "update_goal"}}.get(val, {val})
        arm["tools"] = [t for t in arm["tools"] if t.get("function", t)["name"] not in names]
    elif key == "prompt":
        msgs[0]["content"] = oc_sections()[0] if val == "oc" else msgs[0]["content"]
    elif key == "sec":
        # `sec:# Personality`, `sec:## Autonomy and persistence`
        msgs[0]["content"] = _drop_section(msgs[0]["content"], val)
    elif key == "layout":
        # Everything the fork spreads over four messages, folded into OpenCode's two. Tests whether
        # what costs is the text or the role boundary 30 KB upstream of the task.
        task = msgs[-1]
        joined = "\n\n".join(m["content"] for m in msgs[:-1])
        arm["messages"] = [{"role": "system", "content": joined}, task]
    elif key == "cap":
        arm["policy"]["cap"] = val            # suf | flat | oc
    elif key == "label":
        arm["policy"]["label"] = val          # const | honest
    elif key == "disclose":
        arm["policy"]["disclose"] = val       # total | more | silent
    elif key == "envelope":
        arm["policy"]["envelope"] = val       # suf | oc
    elif key == "noclip":
        arm["policy"]["clip_note"] = False
    elif key == "parallel":
        arm["parallel"] = (val == "on")
    elif key == "counts":
        # Positive control. Measured twice before at 0/56 and 0/74 in band: handed the length of
        # every file up front, this model stops choosing windows altogether. If that does not happen
        # here, the batch is not measuring anything and its other rows mean nothing.
        cid = "call_probe_linecounts_0001"
        names = {t.get("function", t)["name"] for t in arm["tools"]}
        shell = "exec_command" if "exec_command" in names else "bash"
        cmd = ("Get-ChildItem -Recurse codex-api\\src -Filter *.rs | ForEach-Object "
               "{ '{0}\t{1}' -f $_.FullName, (Get-Content $_.FullName | "
               "Measure-Object -Line).Lines }")
        call = {"id": cid, "type": "function",
                "function": {"name": shell, "arguments": json.dumps({"cmd": cmd})}}
        arm["messages"] = arm["messages"] + [
            {"role": "assistant", "content": None, "tool_calls": [call]},
            {"role": "tool", "tool_call_id": cid, "content": _line_count_listing()},
        ]
    elif key == "task":
        arm["messages"][-1]["content"] = TASKS[val]
    else:
        raise SystemExit(f"unknown mutation {spec!r}")
    return arm


def build(name):
    base, *muts = name.split("+")
    arm = base_arm(base)
    arm["name"] = name
    for m in muts:
        apply_mut(arm, m)
    return arm


# --------------------------------------------------------------------------- the loop


NOOP = {
    "todowrite": "Todos updated", "update_plan": "Plan updated", "skill": "No skill loaded",
    "view_image": "Image attached", "get_goal": "No goal set", "create_goal": "Goal created",
    "update_goal": "Goal updated", "write_stdin": "(no output)", "webfetch": "Fetch disabled",
    "request_user_input": "The user did not respond; continue with your best judgement.",
    "task": "Subagent disabled in this environment; do the work yourself.",
}


def run_tool(name, args, arm):
    if name == "read":
        return read_tool(WORK, args.get("filePath", ""), args.get("offset"),
                         args.get("limit"), arm["policy"])
    if name == "glob":
        return T.glob_tool(WORK, args.get("pattern", ""), args.get("path")), None
    if name == "grep":
        return T.grep_tool(WORK, args.get("pattern", ""), args.get("path"), args.get("include")), None
    if name in ("bash", "exec_command"):
        return T.shell_tool(WORK, args.get("command") or args.get("cmd") or ""), None
    if name in ("write", "edit", "apply_patch"):
        return "Success.", None
    return NOOP.get(name, f"{name}: not available in this environment"), None


def chat(client, url, body):
    content, reasoning, calls, usage = "", "", {}, None
    with client.stream("POST", url, json=body) as resp:
        if resp.status_code != 200:
            raise RuntimeError(f"HTTP {resp.status_code}: {resp.read()[:300]!r}")
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
            for ch in obj.get("choices") or []:
                d = ch.get("delta") or {}
                content += d.get("content") or ""
                reasoning += d.get("reasoning_content") or ""
                for tc in d.get("tool_calls") or []:
                    slot = calls.setdefault(tc.get("index", 0), {"id": "", "name": "", "args": ""})
                    if tc.get("id"):
                        slot["id"] = tc["id"]
                    fn = tc.get("function") or {}
                    if fn.get("name"):
                        slot["name"] = fn["name"]
                    slot["args"] += fn.get("arguments") or ""
    return content, reasoning, [calls[k] for k in sorted(calls)], usage


def run_arm(arm, url, rounds, client):
    messages = copy.deepcopy(arm["messages"])
    fresh = cached = out_tok = 0
    reads, rounds_log = [], []

    for step in range(1, rounds + 1):
        body = {"model": "glm-5.3-flash", "messages": messages, "tools": arm["tools"],
                "tool_choice": "auto", "stream": True, "stream_options": {"include_usage": True},
                "thinking": {"type": "enabled"}, "reasoning_effort": "high", "max_tokens": 32_000}
        if arm.get("parallel") is not None:
            body["parallel_tool_calls"] = arm["parallel"]

        content, reasoning, calls, usage = chat(client, url, body)
        if usage:
            pt = usage.get("prompt_tokens", 0)
            ct = (usage.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
            fresh += pt - ct
            cached += ct
            out_tok += usage.get("completion_tokens", 0)

        # Saved, because the diagnosis lives in the model's own words and every batch before this
        # one threw them away.
        rounds_log.append({"step": step, "reasoning": reasoning, "text": content,
                           "calls": [c["name"] for c in calls], "batch": len(calls)})
        if not calls:
            break

        messages.append({"role": "assistant", "content": content or None,
                         "reasoning_content": reasoning or None,
                         "tool_calls": [{"id": c["id"], "type": "function",
                                         "function": {"name": c["name"], "arguments": c["args"]}}
                                        for c in calls]})
        for c in calls:
            try:
                args = json.loads(c["args"] or "{}")
            except ValueError:
                args = {}
            result, meta = run_tool(c["name"], args, arm)
            if meta:
                meta.update(bytes=len(result), step=step)
                meta["frac"] = round(meta["kept"] / max(1, meta["file_lines"]), 3)
                reads.append(meta)
            messages.append({"role": "tool", "tool_call_id": c["id"], "content": result})

    cost = fresh * PRICE["fresh"] + cached * PRICE["cached"] + out_tok * PRICE["out"]
    return {"arm": arm["name"], "rounds": len(rounds_log), "fresh": fresh, "cached": cached,
            "out": out_tok, "cost": round(cost, 5), "reads": reads, "log": rounds_log}


def habit(rows):
    """The headline: what a read drags in, and how much of the file it took."""
    reads = [r for row in rows for r in row["reads"]]
    if not reads:
        return dict(n=0, bytes=0, frac=0, lim="-", bounded="0/0", batch=0)
    lims = [r["limit"] for r in reads]
    set_lims = [x for x in lims if x]
    batches = [s["batch"] for row in rows for s in row["log"] if s["batch"]]
    return dict(
        n=len(reads),
        bytes=round(statistics.median(r["bytes"] for r in reads)),
        frac=round(statistics.median(r["frac"] for r in reads), 2),
        lim=(round(statistics.median(set_lims)) if set_lims else "-"),
        bounded=f"{len(set_lims)}/{len(lims)}",
        batch=round(statistics.median(batches), 1) if batches else 0,
    )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", action="append", default=[],
                    help="base[+mut[+mut...]], e.g. suf+tools:oc+cap:flat")
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--rounds", type=int, default=8)
    ap.add_argument("--port", type=int, default=8799)
    ap.add_argument("--budget", type=float, default=0.15, help="stop the batch at this many dollars")
    ap.add_argument("--out", default=os.path.join(HERE, "probe.jsonl"))
    ap.add_argument("--dry", action="store_true", help="build the arms and print their shape only")
    args = ap.parse_args()

    if not args.arm:
        raise SystemExit("name at least one --arm")
    arms = [build(a) for a in args.arm]

    for a in arms:
        print("  %-38s msgs=%-2d tools=%-3d chars=%-8s policy=%s" % (
            a["name"], len(a["messages"]), len(a["tools"]),
            f'{sum(len(m.get("content") or "") for m in a["messages"]):,}',
            ",".join(f"{k}={v}" for k, v in a["policy"].items() if k != "clip_note")))
    if args.dry:
        return 0

    if os.path.isdir(WORK):
        shutil.rmtree(WORK, ignore_errors=True)
        time.sleep(0.3)
    shutil.copytree(SEED, WORK)

    url = f"http://127.0.0.1:{args.port}/api/paas/v4/chat/completions"
    client = httpx.Client(timeout=httpx.Timeout(600.0, connect=30.0))
    rows = {a["name"]: [] for a in arms}
    spent = 0.0

    # Round-robin. Blocks put a false 29% -> 93% on a variable that did nothing, because something
    # drifts within a session and lands entirely on whichever arm ran first.
    for rep in range(1, args.reps + 1):
        for a in arms:
            if spent >= args.budget:
                print(f"\n!! budget {args.budget:.3f} reached at rep {rep}; stopping")
                rep = -1
                break
            try:
                row = run_arm(a, url, args.rounds, client)
            except Exception as exc:  # noqa: BLE001 - a transport hiccup is not a result
                print(f"  rep {rep:2d}  {a['name']:<38} DROPPED ({type(exc).__name__})", flush=True)
                continue
            spent += row["cost"]
            rows[a["name"]].append(row)
            h = habit([row])
            print(f"  rep {rep:2d}  {a['name']:<38} {row['rounds']}r {h['n']:>3} reads  "
                  f"B/read {h['bytes']:>6}  frac {h['frac']:>4}  lim {h['lim']:>5}  "
                  f"bounded {h['bounded']:>7}  ${row['cost']:.4f}", flush=True)
            with io.open(args.out, "a", encoding="utf-8") as fh:
                fh.write(json.dumps(row) + "\n")
        if rep == -1:
            break

    print()
    print(f"{'arm':38} {'reps':>4} {'reads':>6} {'B/read':>8} {'frac':>6} {'limit':>6} {'bounded':>9} {'batch':>6} {'cost':>8}")
    for a in arms:
        rs = rows[a["name"]]
        if not rs:
            continue
        h = habit(rs)
        print(f"{a['name']:38} {len(rs):>4} {h['n']:>6} {h['bytes']:>8} {h['frac']:>6} "
              f"{h['lim']:>6} {h['bounded']:>9} {h['batch']:>6} ${sum(r['cost'] for r in rs):>7.4f}")
    print(f"\nspent ${spent:.4f} of ${args.budget:.2f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
