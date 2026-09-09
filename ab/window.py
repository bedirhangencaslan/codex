"""What is actually IN the context window when it fills up?

The ledger answers "where did the dollars go", which weights an item by how long it stayed.
This answers a different question: at the moment the window is largest, what occupies it?
That is the number that decides whether shrinking reads buys a denser 60k window.
"""
import sys, collections
sys.path.insert(0, r'C:\Users\Bedirhan\Desktop\agent\ab')
import tiktoken
from ablib import load_home, cmd_of, classify, text_of
from ledger import _kind

enc = tiktoken.get_encoding('o200k_base')
homes = sys.argv[1:] or ['real']

for home in homes:
    for s in load_home(home, n=None, min_requests=20):
        carried = []
        fixed = None
        peak, peak_at, peak_snapshot = 0, None, []
        for req in s.requests:
            calls = {p.get('call_id'): p for p, _ in req.calls()}
            seen = sum(n for _, n in carried)
            if fixed is None:
                fixed = max(0, req.input - seen)
            if req.input > peak:
                peak, peak_at, peak_snapshot = req.input, req.idx, list(carried)
            if req.compaction:
                carried = []
            for payload, _ in req.items:
                t = payload.get('type')
                if t in ('function_call_output', 'custom_tool_call_output'):
                    call = calls.get(payload.get('call_id'))
                    nm, cmd = cmd_of(call) if call else ('?', '')
                    kind = classify(nm, cmd) if call else '?'
                    carried.append(('output: ' + kind, len(enc.encode(text_of(payload.get('output'))))))
                elif t in ('function_call', 'custom_tool_call'):
                    nm, cmd = cmd_of(payload)
                    b = payload.get('input') or cmd or ''
                    carried.append(('call: ' + classify(nm, cmd), len(enc.encode(b))))
                else:
                    bucket, n = _kind(payload, calls)
                    if bucket and n:
                        carried.append((bucket, n))

        if peak < 20000:
            continue
        agg = collections.Counter()
        for bucket, n in peak_snapshot:
            agg[bucket] += n
        agg['fixed prefix'] = fixed
        total = sum(agg.values())
        print('\n=== %s  %s' % (home, s.path.name if hasattr(s.path, "name") else str(s.path)[-46:]))
        print('peak window %d tokens at request %d (%d requests, %d compaction)'
              % (peak, peak_at, len(s.requests), sum(1 for r in s.requests if r.compaction)))
        for bucket, n in agg.most_common(10):
            print('   %-42s %7d  %5.1f%%' % (bucket, n, 100 * n / total))
        reads = sum(n for b, n in agg.items() if b in ('output: read', 'output: list', 'output: search'))
        print('   %-42s %7d  %5.1f%%   <- what a summarizer could touch'
              % ('[reads + listings]', reads, 100 * reads / total))
