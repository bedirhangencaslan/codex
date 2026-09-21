#!/usr/bin/env python3
r"""Strip Rust comments and strings without eating code.

A regex cannot do this, and the one in verify-names.py gets it wrong in a way that hides
findings rather than inventing them: `'(?:[^'\\]|\\.)*'` is meant to match a char literal,
but in Rust a `'` is far more often a lifetime. Given

    fn f<'a>(x: &'a str) -> &'static str

the regex matches from `'a` to `'static` and deletes the real code in between. Anything
analysing the result then sees a file with holes in it and reports nothing.

So this scans properly. Four things a regex cannot do:

    raw strings        r"..", r#".."#, r##".."## - closed by a quote and the same run of #
    nested comments    Rust nests /* /* */ */; a non-greedy match closes at the first */
    lifetimes          `'a` is not a string; `'x'` is. Distinguished by what follows
    byte/C strings     b"..", br#".."#, c".."

Returns the file as (line_number, code_only_text) so findings keep their line numbers.
"""
import re

IDENT_START = re.compile(r"[A-Za-z_]")

# Jump between the things that matter instead of walking every character: on a tree this
# size a per-character loop is minutes, and the text between two delimiters is copied
# verbatim anyway.
EVENT = re.compile(r'(?:b|c|br|rb)?r(#*)"|(?:b|c)?"|//|/\*|\n|\'')
CEVENT = re.compile(r"/\*|\*/|\n")


def scan(src):
    out, buf = [], []
    line_no, i, n = 1, 0, len(src)

    def endline():
        out.append((line_no, "".join(buf)))
        del buf[:]

    def skip(a, b):
        """Consume src[a:b] as one opaque token, keeping line numbering honest."""
        nonlocal line_no
        k = src.count("\n", a, b)
        if k:
            endline()
            line_no += 1
            for _ in range(k - 1):
                out.append((line_no, ""))
                line_no += 1

    while i < n:
        m = EVENT.search(src, i)
        if not m:
            buf.append(src[i:])
            break
        buf.append(src[i:m.start()])
        tok = m.group(0)
        start, i = m.start(), m.end()

        if tok == "\n":
            endline()
            line_no += 1
        elif tok == "//":
            j = src.find("\n", i)
            if j < 0:
                break
            endline()
            line_no += 1
            i = j + 1
        elif tok == "/*":
            depth = 1
            while depth and i < n:
                cm = CEVENT.search(src, i)
                if not cm:
                    i = n
                    break
                t2, i = cm.group(0), cm.end()
                if t2 == "/*":
                    depth += 1
                elif t2 == "*/":
                    depth -= 1
                else:
                    endline()
                    line_no += 1
        elif m.group(1) is not None:                 # raw string, closed by "###
            close = '"' + m.group(1)
            j = src.find(close, i)
            j = n if j < 0 else j
            skip(i, j)
            i = j + len(close)
            buf.append('""')
        elif tok.endswith('"'):                      # ordinary / byte / C string
            j = i
            while j < n:
                if src[j] == "\\":
                    j += 2
                    continue
                if src[j] == '"':
                    break
                j += 1
            skip(i, j)
            i = min(j + 1, n)
            buf.append('""')
        else:                                        # a tick: lifetime or char literal
            rest = src[i:i + 3]
            if rest.startswith("\\"):
                j = src.find("'", i + 1)
                i = (j + 1) if j >= 0 else i
                buf.append("''")
            elif len(rest) >= 2 and rest[1] == "'":
                i += 2
                buf.append("''")
            else:
                buf.append("'")                      # lifetime: keep it, it is code
    endline()
    return out


def code_text(src):
    return "\n".join(t for _, t in scan(src))


def test_spans(src, lines=None):
    """[(start_line, end_line)] of `#[cfg(test)]` / `#[test]` items, brace-matched on code.

    Brace counting has to run on the scanned text: a `{` inside a string or a comment
    would otherwise end the span in the wrong place. Pass `lines` to reuse a scan.
    """
    lines = scan(src) if lines is None else lines
    spans = []
    i = 0
    while i < len(lines):
        _, text = lines[i]
        t = text.strip()
        if t.startswith("#[cfg(test)]") or t.startswith("#[test]") or \
                t.startswith("#[tokio::test") or t.startswith("#[rstest"):
            depth, j, started = 0, i, False
            while j < len(lines):
                depth += lines[j][1].count("{") - lines[j][1].count("}")
                if "{" in lines[j][1]:
                    started = True
                if started and depth <= 0:
                    break
                j += 1
            spans.append((lines[i][0], lines[min(j, len(lines) - 1)][0]))
            i = j + 1
            continue
        i += 1
    return spans
