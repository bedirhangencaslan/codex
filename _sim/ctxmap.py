"""What the model sees on every single request, broken into named blocks.

`CONTEXT-KARSILASTIRMA.md` compared the two agents' *first* requests. The window is not chosen
there - it is chosen on the second and every one after, where the conversation has grown - and the
composition of those requests has never been written down for either agent.

This labels every message of every captured request body into one of a few groups, so the question
becomes arithmetic: at the moment the model picks a window, how many tokens of each kind is it
looking at, and which group differs between the agents.

Groups:
  prompt        the agent's own system prompt
  skills        a skills / permissions block sent as its own system message
  agents_md     AGENTS.md, wherever it is carried
  task          the user's request
  tools         the serialized tool specs
  assistant     the model's own previous turns (text + reasoning + tool call arguments)
  tool_out      what the tools answered
  other         anything unclassified, printed so it cannot hide

    python ctxmap.py <bodies-dir> [--label NAME] [--json out.json]
"""

import argparse
import io
import json
import os
import sys

# ~3.6 characters a token measured against this corpus and this tokenizer's own `usage` numbers;
# used only to make the blocks comparable, never to price anything.
CHARS_PER_TOKEN = 3.6


def load(p):
    return json.loads(io.open(p, encoding="utf-8", errors="replace").read())


def classify(msg, idx, agent):
    """Name the block a message belongs to, from its role, position and its own first line."""
    role = msg.get("role")
    c = msg.get("content")
    text = c if isinstance(c, str) else json.dumps(c, ensure_ascii=False) if c else ""

    if role == "system":
        if "<skills_instructions>" in text or "permissions instructions" in text:
            return "skills"
        # OpenCode carries prompt, env, AGENTS.md and skills in one system message; split it so the
        # two agents' blocks are comparable rather than one being a single 32 KB lump.
        return "prompt"
    if role == "user":
        if "AGENTS.md instructions for" in text or "Instructions from:" in text or "<INSTRUCTIONS>" in text:
            return "agents_md"
        return "task"
    if role == "assistant":
        return "assistant"
    if role == "tool":
        return "tool_out"
    return "other"


def split_opencode_system(text):
    """OpenCode's one system message, cut into prompt / agents_md / skills."""
    out = {}
    i_a = text.find("Instructions from: ")
    i_s = text.find("Skills provide specialized instructions")
    if i_a < 0:
        return {"prompt": text}
    out["prompt"] = text[:i_a]
    end = i_s if i_s > i_a else len(text)
    out["agents_md"] = text[i_a:end]
    if i_s > i_a:
        out["skills"] = text[i_s:]
    return out


def measure(path, agent):
    b = load(path)
    blocks = {}

    def add(name, n):
        blocks[name] = blocks.get(name, 0) + n

    for i, m in enumerate(b.get("messages") or []):
        c = m.get("content")
        text = c if isinstance(c, str) else (json.dumps(c, ensure_ascii=False) if c else "")
        extra = ""
        if m.get("tool_calls"):
            extra += json.dumps(m["tool_calls"], ensure_ascii=False)
        if m.get("reasoning_content"):
            extra += m["reasoning_content"]
        group = classify(m, i, agent)
        if group == "prompt" and m.get("role") == "system":
            for k, v in split_opencode_system(text).items():
                add(k, len(v))
            add("assistant", len(extra))
            continue
        add(group, len(text) + len(extra))

    add("tools", len(json.dumps(b.get("tools") or [], ensure_ascii=False)))
    return blocks, len(b.get("messages") or [])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--label", default="")
    ap.add_argument("--json", default="")
    args = ap.parse_args()

    files = sorted(f for f in os.listdir(args.dir) if f.endswith(".json") and "headers" not in f)
    label = args.label or os.path.basename(os.path.dirname(args.dir.rstrip("\\/")))

    order = ["prompt", "skills", "agents_md", "task", "tools", "assistant", "tool_out", "other"]
    rows = []
    print("=== %s : %d requests ===" % (label, len(files)))
    print("%-5s %-5s %8s %7s %9s %7s %8s %10s %10s %7s %10s" %
          ("req", "msgs", "prompt", "skills", "agents_md", "task", "tools", "assistant", "tool_out", "other", "TOTAL"))
    for f in files:
        blocks, nmsg = measure(os.path.join(args.dir, f), label)
        # A side call - title generation, summarisation - carries no tool surface and is not part of
        # the agent loop. An empty `tools` list still serialises to two characters, so test for real
        # content rather than truthiness.
        if blocks.get("tools", 0) < 100:
            continue
        total = sum(blocks.values())
        rows.append({"file": f, "messages": nmsg, "chars": blocks, "total": total})
        print("%-5s %-5d %8d %7d %9d %7d %8d %10d %10d %7d %10d" %
              (f[:3], nmsg, *[blocks.get(k, 0) for k in order], total))

    if not rows:
        print("no agent-loop requests found")
        return 1

    first, last = rows[0], rows[-1]
    fixed = {k: first["chars"].get(k, 0) for k in ("prompt", "skills", "agents_md", "task", "tools")}
    print()
    print("fixed prefix (resent every request): %s chars = %s" %
          (f"{sum(fixed.values()):,}", " + ".join("%s %s" % (k, f"{v:,}") for k, v in fixed.items() if v)))
    print("grows to:  assistant %s, tool_out %s by request %s" %
          (f"{last['chars'].get('assistant', 0):,}", f"{last['chars'].get('tool_out', 0):,}", last["file"][:3]))
    print("total first -> last: %s -> %s chars  (~%s -> ~%s tokens)" %
          (f"{first['total']:,}", f"{last['total']:,}",
           f"{int(first['total'] / CHARS_PER_TOKEN):,}", f"{int(last['total'] / CHARS_PER_TOKEN):,}"))

    if args.json:
        json.dump({"label": label, "rows": rows}, io.open(args.json, "w", encoding="utf-8"), indent=2)
        print("wrote", args.json)
    return 0


if __name__ == "__main__":
    sys.exit(main())
