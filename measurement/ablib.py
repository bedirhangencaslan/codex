"""Shared loaders for the A/B measurement tools.

Everything is read from rollouts (`<home>/sessions/**/rollout-*.jsonl`) plus, when the fork's
`request_stats` feature was on, the per-request sidecar (`<home>/analytics/<thread>.jsonl`).
Nothing here is part of the product; it only reads what the product already writes.

Prices are the Z.ai `glm-5.3-flash` promo rates in dollars per million tokens.
"""
import glob
import json
import os
import re
from datetime import datetime

FRESH, CACHED, OUTPUT = 0.075, 0.015, 0.25
CACHED_RATIO = CACHED / FRESH  # a cached token costs a fifth of a fresh one

HOMES = {
    'base': '~/.codex-baseline',
    'fork': '~/.suffice-ab',
    'nomem': '~/.suffice-nomem',
    'real': '~/.suffice',
}

SHRINK_MARK = re.compile(r'the harness trimmed this output, the command did not\. (\d+) middle lines \(~(\d+) tokens\) removed')
EXIT_CODE = re.compile(r'(?:Process exited with code|Exit code:)\s*(-?\d+)')


def text_of(content):
    if isinstance(content, str):
        return content
    return ''.join(c.get('text', '') for c in (content or []) if isinstance(c, dict))


def est_tokens(text):
    return len(text) // 4


def parse_ts(ts):
    try:
        return datetime.fromisoformat(ts.replace('Z', '+00:00'))
    except Exception:
        return None


def cmd_of(payload):
    """(tool name, command text) of a tool call, '' when the tool takes no command."""
    try:
        args = json.loads(payload.get('arguments') or '{}')
    except ValueError:
        return payload.get('name') or '?', ''
    if not isinstance(args, dict):
        return payload.get('name') or '?', ''
    cmd = args.get('cmd') or args.get('command') or args.get('chars') or ''
    if isinstance(cmd, list):
        cmd = ' '.join(cmd)
    return payload.get('name') or '?', cmd


def classify(name, cmd):
    c = cmd.lower()
    if name == 'apply_patch' or 'apply_patch' in c:
        return 'patch'
    if name in ('write_stdin',):
        return 'stdin'
    # The `read` tool is a read like any other. Bucketing it as `tool:read` hid every byte it
    # produced from the reads-and-listings share, which is the number that answers whether the
    # window got denser after a shell-to-tool migration.
    if name == 'read':
        return 'read'
    if name not in ('exec_command', 'shell', 'local_shell', 'container.exec', 'unified_exec'):
        return 'tool:' + name
    if re.search(r"set-content|out-file|\[system\.io\.file\]::write|@'|add-content|new-item", c):
        return 'write'
    if re.search(r'\b(get-content|cat|type|sed -n|head|tail)\b', c):
        return 'read'
    if re.search(r'\b(rg|grep|select-string|findstr)\b', c):
        return 'search'
    if re.search(r'\b(get-childitem|ls|dir|find|tree)\b', c):
        return 'list'
    if re.match(r'\s*git\b', c):
        return 'git'
    if re.search(r'\b(python|python3|py|node|cargo|npm|pnpm|pytest|go|dotnet|sqlite3|docker)\b', c):
        return 'run'
    return 'other'


def exit_code(text):
    m = EXIT_CODE.search(text)
    return int(m.group(1)) if m else None


def is_real_user_message(payload):
    if payload.get('type') != 'message' or payload.get('role') != 'user':
        return False
    text = text_of(payload.get('content'))
    return not (text.startswith('<') or text.startswith('# AGENTS.md'))


class Request:
    __slots__ = ('idx', 'ts', 'input', 'cached', 'output', 'reasoning', 'items', 'compaction', 'turn',
                 'new_turn', 'invisible', 'stats')

    def __init__(self, idx, ts, usage, items):
        self.idx = idx
        self.ts = ts
        self.input = usage.get('input_tokens', 0)
        self.cached = usage.get('cached_input_tokens', 0)
        self.output = usage.get('output_tokens', 0)
        self.reasoning = usage.get('reasoning_output_tokens', 0)
        self.items = items
        self.compaction = False
        self.turn = 0
        self.new_turn = False
        self.invisible = any(m.get('invisible_turn') for _, m in items)
        self.stats = None

    @property
    def fresh(self):
        return max(0, self.input - self.cached)

    def cost(self):
        return (self.fresh * FRESH + self.cached * CACHED + self.output * OUTPUT) / 1e6

    def calls(self):
        return [(p, m) for p, m in self.items if p.get('type') in ('function_call', 'custom_tool_call')]

    def outputs(self):
        return [(p, m) for p, m in self.items if p.get('type') in ('function_call_output', 'custom_tool_call_output')]

    def tool_outputs(self):
        """(class, command, output text, est tokens, exit code) for every tool output in this request."""
        calls = {p.get('call_id'): cmd_of(p) for p, _ in self.calls()}
        out = []
        for p, _ in self.outputs():
            text = text_of(p.get('output'))
            name, cmd = calls.get(p.get('call_id'), ('?', ''))
            out.append((classify(name, cmd), cmd, text, est_tokens(text), exit_code(text)))
        return out


class Session:
    def __init__(self, path, home):
        self.path = path
        self.home = home
        self.id = os.path.basename(path)[8:27]
        self.thread_id = None
        self.cwd = ''
        self.source = ''
        self.thread_source = ''
        self.requests = []
        self.compactions = 0
        self.load()

    @property
    def task(self):
        base = os.path.basename(self.cwd.replace('\\', '/').rstrip('/'))
        for side in ('base', 'fork', 'nomem'):
            if base == side:
                return 'sepet'
            if base.startswith(side + '-'):
                return base[len(side) + 1:]
        return base or '?'

    def load(self):
        cur, last_total, idx = [], None, 0
        for line in open(self.path, encoding='utf-8'):
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except ValueError:
                continue
            t, p = r.get('type'), r.get('payload') or {}
            if t == 'session_meta':
                self.thread_id = p.get('id')
                self.cwd = p.get('cwd') or ''
                self.source = str(p.get('source') or '')
                self.thread_source = str(p.get('thread_source') or '')
            elif t == 'response_item':
                cur.append((p, r.get('metadata') or {}))
            elif t == 'event_msg' and p.get('type') == 'token_count':
                info = p.get('info') or {}
                usage, total = info.get('last_token_usage'), info.get('total_token_usage')
                if not usage:
                    continue
                key = json.dumps(total, sort_keys=True)
                if key == last_total:  # task-complete re-snapshot, not a request
                    continue
                last_total = key
                self.requests.append(Request(idx, parse_ts(r.get('timestamp', '')), usage, cur))
                idx += 1
                cur = []
            elif t == 'compacted':
                self.compactions += 1
                if self.requests:
                    self.requests[-1].compaction = True
        turn = 0
        for req in self.requests:
            if any(is_real_user_message(p) for p, _ in req.items):
                turn += 1
                req.new_turn = True
            req.turn = turn

    def attach_stats(self, analytics_dir):
        if not self.thread_id:
            return False
        path = os.path.join(analytics_dir, self.thread_id + '.jsonl')
        if not os.path.exists(path):
            return False
        rows = []
        for line in open(path, encoding='utf-8'):
            try:
                row = json.loads(line)
            except ValueError:
                continue
            if 'sequence' in row:
                rows.append(row)
        rows.sort(key=lambda row: row['sequence'])
        if len(rows) != len(self.requests):
            return False
        for req, row in zip(self.requests, rows):
            if row.get('input_tokens') != req.input:
                return False
        for req, row in zip(self.requests, rows):
            req.stats = row
        return True

    # ---- derived
    def cost(self):
        return sum(r.cost() for r in self.requests)

    def totals(self):
        return {
            'requests': len(self.requests),
            'turns': max((r.turn for r in self.requests), default=0),
            'input': sum(r.input for r in self.requests),
            'cached': sum(r.cached for r in self.requests),
            'output': sum(r.output for r in self.requests),
            'reasoning': sum(r.reasoning for r in self.requests),
            'fresh_cost': sum(r.fresh for r in self.requests) * FRESH / 1e6,
            'cached_cost': sum(r.cached for r in self.requests) * CACHED / 1e6,
            'output_cost': sum(r.output for r in self.requests) * OUTPUT / 1e6,
            'compactions': self.compactions,
            'prefix': self.requests[0].input if self.requests else 0,
            'minutes': self.minutes(),
        }

    def minutes(self):
        ts = [r.ts for r in self.requests if r.ts]
        if len(ts) < 2:
            return 0.0
        return (ts[-1] - ts[0]).total_seconds() / 60

    def remaining(self, idx):
        """Requests after `idx` that still carry the same prefix: up to the next compaction."""
        for r in self.requests[idx + 1:]:
            if r.compaction:
                return r.idx - idx
        return len(self.requests) - 1 - idx


def load_home(home, n=None, task=None, min_requests=1):
    root = os.path.expanduser(HOMES.get(home, home))
    files = sorted(glob.glob(os.path.join(root, 'sessions', '*', '*', '*', 'rollout-*.jsonl')), key=os.path.getmtime)
    sessions = []
    analytics = os.path.join(root, 'analytics')
    for f in files:
        s = Session(f, home)
        if len(s.requests) < min_requests:
            continue
        if task and s.task != task:
            continue
        if os.path.isdir(analytics):
            s.attach_stats(analytics)
        sessions.append(s)
    if n:
        sessions = sessions[-n:]
    return sessions


def money(x):
    return f'{x:.4f}'


def pct(a, b):
    return f'{100 * a / b:.1f}%' if b else '-'


def table(headers, rows):
    out = ['| ' + ' | '.join(headers) + ' |', '|' + '---|' * len(headers)]
    for row in rows:
        out.append('| ' + ' | '.join(str(c) for c in row) + ' |')
    return '\n'.join(out)
