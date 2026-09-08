"""Measure what each fork mechanism does, from rollouts alone, with counterfactuals where the
mechanism leaves no direct trace.

    python measure.py all --home real                    # every section, one home
    python measure.py ab --n 3                           # base vs fork, per task, newest n runs a side
    python measure.py keepalive --home real --ttl 600 --interval 420 --budget 4
    python measure.py retention --home real
    python measure.py shrinker --home base               # what the shrinker would have removed
    python measure.py <section> ... --md out.md          # write markdown instead of printing

Sections: overview, ab, cache, shrinker, batching, retention, keepalive, compaction, failures,
memories, prefix. Prices are the Z.ai promo rates ($0.075 fresh / $0.015 cached / $0.25 output
per million tokens); a counterfactual is labelled as such and states its assumptions inline.
Not part of the product.
"""
import argparse
import collections
import os
import sys

from ledger import ledger
from ablib import CACHED, FRESH, OUTPUT, HOMES, SHRINK_MARK, load_home, money, pct, table, text_of

PREMIUM = FRESH - CACHED  # what a re-prefilled token costs over a cached one


def section(title):
    return f'\n## {title}\n'


# ---------------------------------------------------------------- overview
def overview(sessions):
    rows = []
    for s in sessions:
        t = s.totals()
        rows.append([s.home, s.id, s.task, t['requests'], t['turns'], f"{t['input']:,}", pct(t['cached'], t['input']),
                     f"{t['output']:,}", t['compactions'], f"{t['prefix']:,}", money(t['fresh_cost']),
                     money(t['cached_cost']), money(t['output_cost']), f"**{money(s.cost())}**", f"{t['minutes']:.1f}"])
    return section('Sessions') + table(
        ['home', 'session', 'task', 'req', 'turns', 'input', 'hit', 'output', 'compact', 'prefix', 'fresh $',
         'cached $', 'output $', 'total $', 'min'], rows)


# ---------------------------------------------------------------- ab
def ab(n):
    out = [section('A/B by task (newest runs per side)')]
    per_task = collections.defaultdict(lambda: collections.defaultdict(list))
    for side in ('base', 'fork'):
        for s in load_home(side, min_requests=3):
            per_task[s.task][side].append(s)
    rows = []
    for task, sides in sorted(per_task.items()):
        for side in ('base', 'fork'):
            runs = sides[side][-n:]
            if not runs:
                continue
            costs = [s.cost() for s in runs]
            reqs = [len(s.requests) for s in runs]
            calls = [sum(len(r.calls()) for r in s.requests) for s in runs]
            hit = sum(s.totals()['cached'] for s in runs) / max(1, sum(s.totals()['input'] for s in runs))
            per_call = sum(costs) / max(1, sum(calls))
            rows.append([task, side, len(runs), money(sum(costs) / len(costs)), money(min(costs)), money(max(costs)),
                         f'{sum(reqs) / len(reqs):.1f}', f'{100 * hit:.1f}%', money(per_call)])
    out.append(table(['task', 'side', 'n', 'mean $', 'min $', 'max $', 'req', 'hit', '$ / tool call'], rows))
    out.append('\nRun-to-run spread usually exceeds the side difference; read `min`/`max` before `mean`, and '
               '`$ / tool call` when the two sides did different amounts of work.')
    return '\n'.join(out)


# ---------------------------------------------------------------- cache
def cache(sessions):
    loss = collections.Counter()
    events = collections.Counter()
    total = 0.0
    for s in sessions:
        prev = None
        in_invisible = False
        for r in s.requests:
            total += r.cost()
            if prev is not None:
                expect = min(prev.input, r.input)
                lost = max(0, expect - r.cached)
                if lost > 0.2 * expect and lost > 1000:
                    if r.compaction:
                        k = 'compaction request (summariser)'
                    elif prev.compaction:
                        k = 'first request after compaction'
                    elif r.invisible and not in_invisible:
                        k = 'invisible turn start'
                    elif r.new_turn:
                        k = 'turn boundary (user idle)'
                    elif r.idx <= 5 and r.cached == 0:
                        k = 'session warm-up'
                    else:
                        k = 'same-turn miss'
                    loss[k] += lost * PREMIUM / 1e6
                    events[k] += 1
            in_invisible = r.invisible
            prev = r
    rows = [[k, events[k], money(v), pct(v, total)] for k, v in loss.most_common()]
    return section('Cache losses') + table(['where', 'events', 'premium paid $', 'of bill'], rows) + \
        '\nPremium = tokens the previous request had cached but this one re-prefilled, at fresh minus cached price.'


# ---------------------------------------------------------------- shrinker
def shrinker(sessions):
    """Observed: markers in outputs, priced as tokens removed x remaining requests x cached price.
    Counterfactual on any home: outputs the shrinker would have trimmed (allowlisted command, exit 0,
    >= 45 lines), priced the same way, so a base run shows what the feature would have bought."""
    ALLOW = ('git commit', 'git add', 'git push', 'git fetch', 'git pull', 'git merge', 'git rebase', 'git checkout',
             'cargo', 'rustfmt', 'npm', 'pnpm', 'yarn', 'bun', 'tsc', 'jest', 'vitest', 'webpack', 'vite', 'next',
             'eslint', 'prettier', 'pip', 'uv', 'poetry', 'pytest', 'black', 'ruff', 'go ', 'gofmt', 'make', 'cmake',
             'ninja', 'gradle', 'mvn', 'javac', 'docker')
    seen_n = seen_tok = seen_carry = 0
    would_n = would_tok = would_carry = 0
    for s in sessions:
        for r in s.requests:
            remaining = s.remaining(r.idx)
            for cls, cmd, text, toks, ec in r.tool_outputs():
                m = SHRINK_MARK.search(text)
                if m:
                    removed = int(m.group(2))
                    seen_n += 1
                    seen_tok += removed
                    seen_carry += removed * remaining
                    continue
                first = cmd.strip().lower()
                if ec == 0 and text.count('\n') >= 45 and any(first.startswith(a) for a in ALLOW) \
                        and not any(ch in cmd for ch in '|;><`$&'):
                    lines = text.count('\n')
                    est_removed = int(toks * max(0, lines - 35) / max(1, lines))
                    would_n += 1
                    would_tok += est_removed
                    would_carry += est_removed * remaining
    rows = [
        ['observed (marker present)', seen_n, f'{seen_tok:,}', f'{seen_carry:,}', money(seen_carry * CACHED / 1e6 + seen_tok * FRESH / 1e6)],
        ['counterfactual: would have shrunk', would_n, f'{would_tok:,}', f'{would_carry:,}', money(would_carry * CACHED / 1e6 + would_tok * FRESH / 1e6)],
    ]
    return section('Tool-output shrinker') + table(['', 'outputs', 'tokens removed', 'carry tokens', 'saved $'], rows) + \
        '\nSaved = removed tokens once at fresh price plus removed x remaining requests at cached price. ' \
        'The counterfactual keeps head 5 + tail 30 lines and assumes tokens are spread evenly over lines.'


# ---------------------------------------------------------------- batching
def batching(sessions):
    responses = 0
    calls = 0
    saved_rounds = 0
    saved_cost = 0.0
    dist = collections.Counter()
    for s in sessions:
        for r in s.requests:
            k = len(r.calls())
            if k == 0:
                continue
            responses += 1
            calls += k
            dist[min(k, 5)] += 1
            if k > 1:
                saved_rounds += k - 1
                # each avoided round would have re-read the prefix at cached price and produced
                # a short response (~80 output tokens for the call itself)
                saved_cost += (k - 1) * (r.input * CACHED + 80 * OUTPUT) / 1e6
    rows = [[responses, calls, f'{calls / max(1, responses):.2f}', saved_rounds, money(saved_cost),
             ' '.join(f'{k}:{dist[k]}' for k in sorted(dist))]]
    return section('Tool-call batching') + table(
        ['tool responses', 'calls', 'calls / response', 'rounds avoided', 'saved $ (counterfactual)', 'calls per response histogram'], rows) + \
        '\nCounterfactual: every extra call in a response would otherwise have been its own round at the same context size.'


# ---------------------------------------------------------------- retention
def retention(sessions):
    """Per finished turn: R = reasoning the turn produced (provider usage), S = context the turn
    added after its first reasoning (what dropping re-prefills), N = requests left before the next
    compaction. keep = R*N*cached, drop = S*premium. Reports keep-all, drop-all, per-turn optimum,
    and what actually happened where the fork's sidecar recorded it."""
    keep_all = drop_all = optimal = observed = 0.0
    turns = 0
    dropped_events = 0
    observed_known = True
    for s in sessions:
        by_turn = collections.defaultdict(list)
        for r in s.requests:
            if r.turn and not r.invisible and not r.compaction:
                by_turn[r.turn].append(r)
        for t, reqs in sorted(by_turn.items()):
            last = reqs[-1]
            n = s.remaining(last.idx)
            if n <= 0:
                continue
            R = sum(r.reasoning for r in reqs)
            S = max(0, last.input - reqs[0].input)
            keep = R * n * CACHED / 1e6
            drop = S * PREMIUM / 1e6
            turns += 1
            keep_all += keep
            drop_all += drop
            optimal += min(keep, drop)
            nxt = s.requests[last.idx + 1] if last.idx + 1 < len(s.requests) else None
            if nxt is not None and nxt.stats is not None:
                if nxt.stats.get('dropped_reasoning_tokens', 0) > 0:
                    dropped_events += 1
                    observed += nxt.stats.get('prefix_break_tokens', S) * PREMIUM / 1e6
                else:
                    observed += keep
            else:
                observed_known = False
    rows = [['keep every finished turn\'s reasoning', money(keep_all)],
            ['drop at turn end (upstream policy)', money(drop_all)],
            ['per-turn optimum (what the cost model aims at)', money(optimal)],
            [f'observed via sidecar ({dropped_events} drops)' if observed_known else 'observed (sidecar incomplete, partial)', money(observed)]]
    return section(f'Reasoning retention ({turns} finished turns with requests after them)') + \
        table(['policy', 'cost $'], rows) + \
        '\nR from provider usage, S = context added by the turn after its first request, N = requests to the next compaction.'


# ---------------------------------------------------------------- keepalive
def keepalive(sessions, ttl, interval, budget):
    """Only user-idle gaps matter: the keep-alive is held while a tool call runs. A gap is the time
    from a turn's last request to the next turn's first request, minus that request's own duration
    when the sidecar knows it."""
    rows = []
    tot_without = tot_with_cost = tot_with_saved = 0.0
    n_gaps = n_over = 0
    observed_hits = observed_misses = 0
    for s in sessions:
        for r in s.requests:
            if not r.new_turn or r.idx == 0 or r.ts is None:
                continue
            prev = s.requests[r.idx - 1]
            if prev.ts is None or prev.compaction:
                continue
            gap = (r.ts - prev.ts).total_seconds()
            if r.stats and r.stats.get('duration_ms'):
                gap -= r.stats['duration_ms'] / 1000
            if gap <= 0:
                continue
            n_gaps += 1
            prefix = min(prev.input, r.input)
            hit = r.cached >= 0.8 * prefix
            if gap > ttl:
                n_over += 1
                observed_hits += hit
                observed_misses += (not hit)
                tot_without += prefix * PREMIUM / 1e6
                refreshes = min(int(gap // interval), budget)
                covered = refreshes * interval + ttl >= gap
                tot_with_cost += refreshes * prefix * CACHED / 1e6
                if covered:
                    tot_with_saved += prefix * PREMIUM / 1e6
    rows.append([f'{ttl}', f'{interval}', budget, n_gaps, n_over, money(tot_without), money(tot_with_cost), money(tot_with_saved),
                 money(tot_with_saved - tot_with_cost), f'{observed_hits} hit / {observed_misses} miss'])
    return section('Prompt-cache keep-alive (counterfactual)') + table(
        ['ttl s', 'interval s', 'budget', 'idle gaps', 'gaps > ttl', 'lost without $', 'refresh cost $', 'saved with $',
         'net $', 'observed after long gaps'], rows) + \
        '\n`observed` says what actually happened after gaps longer than the TTL in these sessions: hits mean a ' \
        'keep-alive (or a longer real TTL) covered the gap. Measured TTL bounds on Z.ai: hits up to 448 s, miss at 1,023 s.'


# ---------------------------------------------------------------- compaction
def compaction(sessions):
    rows = []
    tot_cost = tot_tools_saving = 0.0
    for s in sessions:
        for r in s.requests:
            if not r.compaction:
                continue
            before = s.requests[r.idx - 1].input if r.idx > 0 else 0
            after = s.requests[r.idx + 1].input if r.idx + 1 < len(s.requests) else 0
            tools_saving = r.fresh * PREMIUM / 1e6  # if the summariser had shared the cached prefix
            tot_cost += r.cost()
            tot_tools_saving += tools_saving
            rows.append([s.id, r.idx, f'{before:,}', f'{r.input:,}', pct(r.cached, r.input), f'{after:,}', money(r.cost()), money(tools_saving)])
    rows.append(['**total**', '', '', '', '', '', f'**{money(tot_cost)}**', f'**{money(tot_tools_saving)}**'])
    return section('Compaction') + table(
        ['session', 'req', 'context before', 'summariser input', 'cached', 'context after', 'cost $', 'if prefix cached $'], rows) + \
        '\n`if prefix cached` is the counterfactual saving of sending the summariser on the same cached prefix (the tool-specs line that was reverted).'


# ---------------------------------------------------------------- failures
def failures(sessions):
    by_class = collections.Counter()
    fail_by_class = collections.Counter()
    repeated = 0
    extra_cost = 0.0
    top = collections.Counter()
    for s in sessions:
        avg = s.cost() / max(1, len(s.requests))
        last_fail = None
        for r in s.requests:
            failed_here = False
            for cls, cmd, text, toks, ec in r.tool_outputs():
                by_class[cls] += 1
                if ec not in (None, 0):
                    fail_by_class[cls] += 1
                    failed_here = True
                    key = cmd.strip()[:70]
                    if key and key == last_fail:
                        repeated += 1
                    last_fail = key
                    top[(cls, key)] += 1
                else:
                    last_fail = None
            if failed_here:
                extra_cost += avg
    rows = [[cls, by_class[cls], fail_by_class[cls], pct(fail_by_class[cls], by_class[cls])] for cls, _ in by_class.most_common()]
    out = section('Tool failures') + table(['class', 'calls', 'failed', 'rate'], rows)
    out += f'\n\nResponses with at least one failure: priced at the session\'s mean request cost = **{money(extra_cost)}** ' \
           f'(upper bound on what the retries cost). Identical failing command retried immediately: {repeated}.\n'
    rep = [[n, cls, cmd] for (cls, cmd), n in top.most_common(8) if n > 1]
    if rep:
        out += '\n' + table(['times', 'class', 'command'], rep)
    return out


# ---------------------------------------------------------------- memories
def memories(home, sessions):
    root = os.path.expanduser(HOMES.get(home, home))
    mem = os.path.join(root, 'memories')
    lines = [section('Memories pipeline')]
    if not os.path.isdir(mem):
        lines.append('No `memories/` directory: the pipeline has not run in this home.')
    else:
        summary = os.path.join(mem, 'memory_summary.md')
        if os.path.exists(summary):
            lines.append(f'`memory_summary.md`: {os.path.getsize(summary):,} bytes, modified {os.path.getmtime(summary):.0f}')
        else:
            lines.append('`memories/` exists but `memory_summary.md` does not: layout prepared, nothing consolidated yet.')
        entries = sorted(os.listdir(mem))
        lines.append('Contents: ' + ', '.join(entries))
    consol = [s for s in sessions if 'memory' in (s.thread_source + s.source).lower()]
    if consol:
        rows = [[s.id, s.thread_source or s.source, len(s.requests), money(s.cost())] for s in consol]
        lines.append(table(['session', 'source', 'req', 'cost $'], rows))
    else:
        lines.append('No consolidation threads found among these sessions.')
    return '\n'.join(lines)


# ---------------------------------------------------------------- prefix
def prefix(sessions):
    rows = []
    for s in sessions:
        first = s.requests[0]
        dev = sum(len(text_of(p.get('content'))) // 4 for p, _ in first.items if p.get('type') == 'message' and p.get('role') == 'developer')
        agents = sum(len(text_of(p.get('content'))) // 4 for p, _ in first.items
                     if p.get('type') == 'message' and p.get('role') == 'user' and text_of(p.get('content')).startswith('# AGENTS.md'))
        carried = sum(min(r.input, first.input) for r in s.requests[1:])
        rows.append([s.id, f'{first.input:,}', f'~{dev:,}', f'~{agents:,}', f'{carried:,}', money(carried * CACHED / 1e6), pct(carried * CACHED / 1e6, s.cost())])
    return section('Fixed prefix') + table(['session', 'first request input', 'developer msgs', 'AGENTS.md', 'prefix re-read (tokens)', 'carry $', 'of bill'], rows) + \
        '\nThe first request is instructions + tool specs + developer messages + AGENTS.md + the prompt; every later request re-reads it at cached price.'


# ---------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('section', choices=['all', 'overview', 'ab', 'ledger', 'cache', 'shrinker', 'batching', 'retention',
                                        'keepalive', 'compaction', 'failures', 'memories', 'prefix'])
    ap.add_argument('--home', default='real', help='base | fork | nomem | real | path')
    ap.add_argument('--n', type=int, default=None, help='newest n sessions only')
    ap.add_argument('--task', default=None)
    ap.add_argument('--ttl', type=int, default=600)
    ap.add_argument('--interval', type=int, default=420)
    ap.add_argument('--budget', type=int, default=4)
    ap.add_argument('--md', default=None)
    a = ap.parse_args()

    if a.section == 'ab':
        out = ab(a.n or 3)
    else:
        sessions = load_home(a.home, a.n, a.task, min_requests=1)
        working = [s for s in sessions if len(s.requests) >= 3]
        parts = [f'# Measurement: {a.home} ({len(working)} sessions)']
        want = [a.section] if a.section != 'all' else ['overview', 'ledger', 'prefix', 'cache', 'compaction', 'retention', 'keepalive',
                                                        'shrinker', 'batching', 'failures', 'memories']
        for sec in want:
            if sec == 'overview':
                parts.append(overview(working))
            elif sec == 'ledger':
                parts.append(ledger(working))
            elif sec == 'cache':
                parts.append(cache(working))
            elif sec == 'shrinker':
                parts.append(shrinker(working))
            elif sec == 'batching':
                parts.append(batching(working))
            elif sec == 'retention':
                parts.append(retention(working))
            elif sec == 'keepalive':
                parts.append(keepalive(working, a.ttl, a.interval, a.budget))
            elif sec == 'compaction':
                parts.append(compaction(working))
            elif sec == 'failures':
                parts.append(failures(working))
            elif sec == 'memories':
                parts.append(memories(a.home, sessions))
            elif sec == 'prefix':
                parts.append(prefix(working))
        out = '\n'.join(parts)
    if a.md:
        with open(a.md, 'w', encoding='utf-8') as f:
            f.write(out + '\n')
        print(f'wrote {a.md}')
    else:
        sys.stdout.reconfigure(encoding='utf-8')
        print(out)


if __name__ == '__main__':
    main()
