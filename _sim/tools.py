"""Local implementations of the tools both agents expose, in both output envelopes.

The point of the simulator is that the *only* thing that differs between two arms is the piece
under test. So every tool is implemented once here and rendered through whichever envelope the arm
asks for, rather than being re-implemented per agent.

Numbers are the ones each agent actually uses, read from its own source:
  glob/grep result cap 100      opencode `glob.ts:49`, `grep.ts:80`; suffice `search.rs:56-57`
  read byte cap                 opencode `read.ts:16` = 50 KB; suffice `read.rs:71` = 32 KB
  read default line limit 2000  both
"""

import io
import os
import re
import subprocess

GLOB_CAP = 100
GREP_CAP = 100
READ_DEFAULT_LIMIT = 2000
MAX_LINE_CHARS = 2000

OC_READ_MAX_BYTES = 50 * 1024
SUF_READ_MAX_BYTES = 32_000


def _text(path):
    return io.open(path, encoding="utf-8", errors="replace").read()


def _walk(root):
    out = []
    for base, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in (".git", "target", "node_modules")]
        for f in files:
            out.append(os.path.join(base, f))
    out.sort()
    return out


# --------------------------------------------------------------------------- glob


def glob_tool(root, pattern, path=None):
    base = os.path.abspath(os.path.join(root, path)) if path else root
    rx = _glob_to_regex(pattern)
    hits = []
    for fp in _walk(base):
        rel = os.path.relpath(fp, root).replace("\\", "/")
        if rx.match(rel) or rx.match(os.path.basename(fp)):
            hits.append(fp)
    shown = hits[:GLOB_CAP]
    if not shown:
        return "No files found"
    out = "\n".join(shown)
    if len(hits) > len(shown):
        out += (
            f"\n\n(Results are truncated: showing first {len(shown)} results. "
            "Consider using a more specific path or pattern.)"
        )
    return out


def _glob_to_regex(pattern):
    p = pattern.replace("\\", "/")
    out, i = "", 0
    while i < len(p):
        if p.startswith("**/", i):
            out += "(?:.*/)?"
            i += 3
        elif p.startswith("**", i):
            out += ".*"
            i += 2
        elif p[i] == "*":
            out += "[^/]*"
            i += 1
        elif p[i] == "?":
            out += "[^/]"
            i += 1
        elif p[i] == "{":
            j = p.index("}", i)
            out += "(?:" + "|".join(re.escape(x) for x in p[i + 1 : j].split(",")) + ")"
            i = j + 1
        else:
            out += re.escape(p[i])
            i += 1
    return re.compile("^" + out + "$")


# --------------------------------------------------------------------------- grep


def grep_tool(root, pattern, path=None, include=None):
    base = os.path.abspath(os.path.join(root, path)) if path else root
    try:
        rx = re.compile(pattern)
    except re.error as exc:
        return f"Invalid regex: {exc}"
    inc = _glob_to_regex(include) if include else None

    per_file, found = [], 0
    for fp in _walk(base):
        if inc and not (inc.match(os.path.basename(fp)) or inc.match(os.path.relpath(fp, root).replace("\\", "/"))):
            continue
        try:
            body = _text(fp)
        except OSError:
            continue
        rows = []
        for n, line in enumerate(body.splitlines(), 1):
            if rx.search(line):
                found += 1
                if found > GREP_CAP:
                    break
                rows.append(f"{n}: {line[:MAX_LINE_CHARS]}")
        if rows:
            per_file.append((fp, rows))
        if found > GREP_CAP:
            break

    if not per_file:
        return "No matches found"
    blocks = [f"{fp}\n" + "\n".join(rows) for fp, rows in per_file]
    out = f"Found {min(found, GREP_CAP)} matches\n\n" + "\n\n".join(blocks)
    if found > GREP_CAP:
        out += "\n\n(Results truncated. Consider using a more specific path or pattern.)"
    return out


# --------------------------------------------------------------------------- read


def read_tool(root, file_path, offset=None, limit=None, envelope="oc"):
    fp = file_path if os.path.isabs(file_path) else os.path.join(root, file_path)
    fp = os.path.abspath(fp)
    if not os.path.isfile(fp):
        return f"File not found: {fp}"

    lines = _text(fp).splitlines()
    total = len(lines)
    start = max(1, offset or 1)
    lim = limit if limit is not None else READ_DEFAULT_LIMIT
    cap = OC_READ_MAX_BYTES if envelope == "oc" else SUF_READ_MAX_BYTES

    kept, size, cut = [], 0, False
    for i in range(start - 1, min(total, start - 1 + lim)):
        text = lines[i]
        if len(text) > MAX_LINE_CHARS:
            text = text[:MAX_LINE_CHARS] + f"... (line truncated to {MAX_LINE_CHARS} chars)"
        row = f"{i + 1}: {text}"
        if size + len(row) + 1 > cap:
            cut = True
            break
        kept.append(row)
        size += len(row) + 1

    last = start + len(kept) - 1
    more = cut or (start - 1 + len(kept) < total)

    if envelope == "oc":
        body = f"<path>{fp}</path>\n<type>file</type>\n<content>\n\n" + "\n".join(kept)
        if cut:
            body += (
                f"\n\n(Output capped at {OC_READ_MAX_BYTES // 1024} KB. Showing lines "
                f"{start}-{last}. Use offset={last + 1} to continue.)"
            )
        elif more:
            body += f"\n\n(Showing lines {start}-{last} of {total}. Use offset={last + 1} to continue.)"
        else:
            body += f"\n\n(End of file - total {total} lines)"
        return body + "\n</content>"

    # suffice, as it was before the imitation commit: silent when the window fitted
    note = ""
    if more:
        note = f" (showing {len(kept)} of {total} lines; continue with offset {last + 1})"
    elif start > 1:
        note = f" (from line {start})"
    return f"===== {fp}{note}\n" + "\n".join(kept) + "\n"


def read_suffice_batch(root, paths, offset=None, limit=None):
    """The pre-`baeb79fc8` batched shape: one call, many files, one shared window."""
    out = []
    for item in paths or []:
        if isinstance(item, dict):
            out.append(
                read_tool(root, item.get("path", ""), item.get("offset", offset),
                          item.get("limit", limit), envelope="suffice")
            )
        else:
            out.append(read_tool(root, item, offset, limit, envelope="suffice"))
    return "\n".join(out)


# --------------------------------------------------------------------------- shell


def shell_tool(root, command, cap_bytes=51_200):
    """Real execution, in the throwaway corpus copy, with the arm's own output cap."""
    try:
        proc = subprocess.run(
            ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command],
            cwd=root, capture_output=True, text=True, timeout=60,
        )
        out = (proc.stdout or "") + (proc.stderr or "")
        code = proc.returncode
    except subprocess.TimeoutExpired:
        return "Command timed out after 60s"
    except OSError as exc:
        return f"Command failed to start: {exc}"

    if len(out) > cap_bytes:
        out = "...output truncated...\n\n" + out[-cap_bytes:]
    return f"Exit code: {code}\n\n{out or '(no output)'}"
