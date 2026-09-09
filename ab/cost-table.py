"""Price the newest N sessions of each home at real Z.ai prices and print one table.

    python cost-table.py [N] [home ...]

Prices are the glm-5.3-flash promo: $0.075 fresh input, $0.015 cached input, $0.25 output,
all per million tokens. Only the provider's billed usage is read.
"""
import glob
import json
import os
import sys
from datetime import datetime

FRESH, CACHED, OUT = 0.075, 0.015, 0.25
HOMES = {'base': '~/.codex-baseline', 'fork': '~/.suffice-ab', 'real': '~/.suffice'}


def load(path):
    reqs, last_total, compactions, first, last = [], None, 0, None, None
    for line in open(path, encoding='utf-8'):
        line = line.strip()
        if not line:
            continue
        try:
            r = json.loads(line)
        except ValueError:
            continue
        t, p = r.get('type'), r.get('payload') or {}
        ts = r.get('timestamp')
        if ts:
            first = first or ts
            last = ts
        if t == 'event_msg' and p.get('type') == 'token_count':
            info = p.get('info') or {}
            u, tot = info.get('last_token_usage'), info.get('total_token_usage')
            if not u:
                continue
            key = json.dumps(tot, sort_keys=True)
            if key == last_total:
                continue
            last_total = key
            reqs.append(u)
        elif t == 'compacted':
            compactions += 1
    return reqs, compactions, first, last


def main():
    args = sys.argv[1:]
    n = int(args[0]) if args and args[0].isdigit() else 1
    homes = [a for a in args if a in HOMES] or ['base', 'fork']
    rows = []
    for home in homes:
        files = sorted(glob.glob(os.path.expanduser(HOMES[home]) + '/sessions/*/*/*/rollout-*.jsonl'), key=os.path.getmtime)
        for f in files[-n:]:
            reqs, comp, first, last = load(f)
            inp = sum(u.get('input_tokens', 0) for u in reqs)
            cached = sum(u.get('cached_input_tokens', 0) for u in reqs)
            out = sum(u.get('output_tokens', 0) for u in reqs)
            reasoning = sum(u.get('reasoning_output_tokens', 0) for u in reqs)
            fresh_cost = (inp - cached) * FRESH / 1e6
            cached_cost = cached * CACHED / 1e6
            out_cost = out * OUT / 1e6
            secs = 0
            try:
                secs = (datetime.fromisoformat(last.replace('Z', '+00:00')) - datetime.fromisoformat(first.replace('Z', '+00:00'))).total_seconds()
            except Exception:
                pass
            rows.append((home, os.path.basename(f)[8:27], len(reqs), inp, cached, out, reasoning, comp, fresh_cost, cached_cost, out_cost, secs))
    print('| side | session | req | input | cached | hit | output | reasoning | compact | fresh $ | cached $ | output $ | **total $** | min |')
    print('|---|---|---|---|---|---|---|---|---|---|---|---|---|---|')
    for home, sid, nreq, inp, cached, out, rs, comp, fc, cc, oc, secs in rows:
        hit = 100 * cached / inp if inp else 0
        print(f'| {home} | {sid} | {nreq} | {inp:,} | {cached:,} | {hit:.1f}% | {out:,} | {rs:,} | {comp} | {fc:.4f} | {cc:.4f} | {oc:.4f} | **{fc+cc+oc:.4f}** | {secs/60:.1f} |')


main()
