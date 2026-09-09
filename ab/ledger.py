"""Splits a session's bill into operations that add back up to the total.

Every other section here measures one mechanism. This one answers the question those cannot:
of the dollars actually spent, how many went to reading files, to thinking, to patch bodies,
to the system prompt. Each request's cost is real (provider usage); only the split of that
cost across the items in its prompt is estimated, from `len(text) // 4`.

Two things make the arithmetic honest:

- **Input is charged where it is carried, not where it was produced.** A patch body written
  once is re-sent on every later request of the window, and the ledger charges it every time.
  That is what makes the shares here differ from a naive "how big was the output".
- **The blended rate is used per request.** The provider says how much of one request was
  cached, but not which items were; splitting the whole request at its own fresh/cached mix
  is the most that can be claimed without inventing a per-item cache map.

`fixed prefix` is what the first request costs beyond the items visible in the rollout -- the
system instructions and the tool specs, which never appear as history items. It is measured
once per session and then charged on every request, because it is re-sent every time.
"""
import collections

from ablib import CACHED, FRESH, OUTPUT, cmd_of, classify, est_tokens, money, table, text_of


def _kind(payload, calls):
    """The ledger bucket for one history item."""
    t = payload.get('type')
    if t in ('function_call', 'custom_tool_call'):
        name, cmd = cmd_of(payload)
        body = payload.get('input') or cmd or ''
        return f'call: {classify(name, cmd)}', est_tokens(body)
    if t in ('function_call_output', 'custom_tool_call_output'):
        call = calls.get(payload.get('call_id'))
        name, cmd = cmd_of(call) if call else ('?', '')
        return f'output: {classify(name, cmd) if call else "?"}', est_tokens(text_of(payload.get('output')))
    if t == 'message':
        role = payload.get('role') or '?'
        text = text_of(payload.get('content'))
        # AGENTS.md and the environment block arrive as user messages but are harness-authored.
        if role == 'user' and (text.startswith('<') or text.startswith('# AGENTS.md')):
            return 'harness context', est_tokens(text)
        return f'message: {role}', est_tokens(text)
    if t == 'reasoning':
        parts = (payload.get('content') or []) + (payload.get('summary') or [])
        text = ''.join(c.get('text') or c.get('summary_text') or '' for c in parts if isinstance(c, dict))
        return 'reasoning', est_tokens(text)
    return None, 0


def session_ledger(session):
    """`{bucket: dollars}` for one session, summing to `session.cost()`."""
    spend = collections.Counter()
    carried = []            # (bucket, tokens) still in the prompt
    fixed = None

    for req in session.requests:
        calls = {p.get('call_id'): p for p, _ in req.calls()}

        # The prompt for this request is everything carried so far. Items recorded during this
        # request were produced by it and are only carried from the next one onwards.
        seen = sum(n for _, n in carried)
        if fixed is None:
            # Whatever the first request paid for beyond visible history is the fixed prefix.
            fixed = max(0, req.input - seen)

        blended = (req.fresh * FRESH + req.cached * CACHED) / 1e6
        billable = seen + fixed
        if billable > 0:
            spend['fixed prefix (instructions + tool specs)'] += blended * fixed / billable
            for bucket, n in carried:
                spend[bucket] += blended * n / billable
        else:
            spend['fixed prefix (instructions + tool specs)'] += blended

        # Output: the provider reports reasoning separately; the rest is what the model wrote.
        out_cost = req.output * OUTPUT / 1e6
        if req.output > 0:
            share = min(req.reasoning, req.output) / req.output
            spend['generated: reasoning'] += out_cost * share
            spend['generated: replies and tool calls'] += out_cost * (1 - share)

        # Compaction replaces history, so what preceded it stops being carried -- but only after
        # this request, which still paid to send it.
        if req.compaction:
            carried = []

        for payload, _ in req.items:
            bucket, n = _kind(payload, calls)
            if bucket and n:
                carried.append((bucket, n))

    return spend


def ledger(sessions):
    total = collections.Counter()
    for s in sessions:
        total.update(session_ledger(s))

    billed = sum(s.cost() for s in sessions)
    grand = sum(total.values())
    rows = []
    for bucket, dollars in sorted(total.items(), key=lambda kv: -kv[1]):
        rows.append((bucket, money(dollars), f'{100 * dollars / grand:.1f}%'))
    rows.append(('**total**', f'**{money(grand)}**', '100.0%'))

    lines = [
        '## Where the money went',
        '',
        table(['operation', '$', 'share'], rows),
        '',
        f'Billed from provider usage: **{money(billed)}** over {sum(len(s.requests) for s in sessions)} '
        f'requests in {len(sessions)} session(s); the ledger reconciles to {money(grand)}.',
        '',
        'Input is charged on every request that carries it, so a large item written once shows up '
        'here multiplied by how long it stayed in the window. `generated:` rows are output tokens, '
        'charged once. `fixed prefix` is the instructions and tool specs, which never appear as '
        'history items and are re-sent every request.',
    ]
    return '\n'.join(lines)
