"""Turn-by-turn accounting for one codex rollout.

The four-way run left Suffice at 27 requests against OpenCode's 11 for the same deliverable. Cost
follows request count almost exactly, because every request re-sends the window - so the question
"why 27" is the whole question. This prints what each request actually bought.
"""

import json
import os
import re
import sys

CAT = [
    ("liste", re.compile(r"rg --files|Get-ChildItem|\bls\b|dir /", re.I)),
    ("arama", re.compile(r"\brg\b|Select-String|findstr", re.I)),
    ("okuma", re.compile(r"Get-Content|\bcat\b|\btype\b|Select-Object -Skip", re.I)),
    ("yazma", re.compile(r"Set-Content|Out-File|Add-Content|>\s*WIRE", re.I)),
]


def label(cmd):
    for name, pat in CAT:
        if pat.search(cmd):
            return name
    return "diger"


def files_touched(cmd):
    return len(re.findall(r"[\w./\\-]+\.rs", cmd))


def main(path):
    items, reqs = [], []
    for line in open(path, encoding="utf-8"):
        r = json.loads(line)
        t = r.get("type")
        if t == "response_item":
            items.append(r["payload"])
        elif t == "event_msg" and (r.get("payload") or {}).get("type") == "token_count":
            reqs.append(items)
            items = []
        elif t == "compacted":
            if reqs:
                reqs[-1] = reqs[-1] + [{"type": "__compacted__"}]

    totals = {}
    read_calls = []
    print(f"{'req':>4}  {'ne yapti':<48} dosya")
    for i, it in enumerate(reqs):
        acts, n = [], 0
        for p in it:
            if p.get("type") == "__compacted__":
                acts.append("COMPACTION")
                continue
            if p.get("type") not in ("function_call", "custom_tool_call"):
                continue
            if p.get("name") == "apply_patch":
                acts.append("apply_patch(WIRE.md)")
                totals["yazma"] = totals.get("yazma", 0) + 1
                continue
            try:
                a = json.loads(p.get("arguments") or "{}")
            except ValueError:
                a = {}
            # The `read` tool is not a shell command, so the regexes below never matched it and
            # every call landed in `diger`. Counting its entries is the whole point of the dump: a
            # batch that collapses into single-path calls is the failure this tool was built for.
            if p.get("name") == "read":
                entries = a.get("paths") or a.get("files") or []
                if isinstance(entries, (str, dict)):
                    entries = [entries]
                windows = sum(
                    1
                    for e in entries
                    if isinstance(e, dict) and ("offset" in e or "limit" in e)
                )
                if a.get("offset") is not None or a.get("limit") is not None:
                    windows = windows or len(entries)
                read_calls.append((len(entries), windows, a.get("offset") is not None))
                n += len(entries)
                acts.append(
                    "read(%d dosya%s)"
                    % (len(entries), ", %d pencere" % windows if windows else "")
                )
                totals["okuma"] = totals.get("okuma", 0) + 1
                continue
            cmd = a.get("cmd") or a.get("command") or ""
            if isinstance(cmd, list):
                cmd = " ".join(cmd)
            k = label(cmd)
            f = files_touched(cmd)
            n += f
            acts.append(f"{k}({f} dosya)" if f else k)
            totals[k] = totals.get(k, 0) + 1
        if not acts:
            msgs = [p for p in it if p.get("type") == "message"]
            acts = ["(mesaj/ozet turu)"] if msgs else ["(bos)"]
        print(f"{i:>4}  {', '.join(acts)[:48]:<48} {n if n else ''}")
    print("\nkategori toplamlari:", totals)
    print("toplam istek:", len(reqs))
    if read_calls:
        sizes = [n for n, _, _ in read_calls]
        with_window = sum(1 for _, w, _ in read_calls if w)
        # A single-path call carrying an offset is the window-chasing signature: the model wanted a
        # window and one file per call was the only way to ask for one.
        chasing = sum(1 for n, _, off in read_calls if n <= 1 and off)
        print("read cagrilari:", sizes)
        print(
            "  toplam dosya=%d  pencere tasiyan cagri=%d/%d  offsetli tek-yollu=%d"
            % (sum(sizes), with_window, len(read_calls), chasing)
        )


if __name__ == "__main__":
    sys.exit(main(os.path.expanduser(sys.argv[1])))
