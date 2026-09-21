#!/usr/bin/env python3
r"""Reading whole revisions out of git without paying for a process per file.

On Windows a subprocess costs 25-35 ms, so `git show rev:path` in a loop over a few
thousand files is minutes of pure spawn overhead. `cat-file --batch` answers all of them
in one process, and `ls-tree -r` gives the blob ids to ask for.
"""
import os
import subprocess
import threading

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=HERE,
                      capture_output=True, encoding="utf-8").stdout.strip()


def git(*args):
    return subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True,
                          encoding="utf-8", errors="replace").stdout


def tree(rev, suffix=None):
    """path -> blob sha for a whole revision, in one process."""
    out = {}
    for line in git("ls-tree", "-r", rev).splitlines():
        meta, _, path = line.partition("\t")
        parts = meta.split()
        if len(parts) >= 3 and parts[1] == "blob":
            if suffix is None or path.endswith(suffix):
                out[path] = parts[2]
    return out


def batch_blobs(shas):
    """sha -> bytes, in one `cat-file --batch`.

    The stream is `<sha> blob <size>\\n`, then exactly <size> bytes, then `\\n`. The
    payload is read by byte count: blobs contain newlines and binary files contain
    anything, so readline() on the payload would desynchronise the stream.

    stdin is fed from a thread. The request list is bigger than a pipe buffer and git
    starts answering immediately, so writing it all before reading deadlocks - we block
    on a full stdin buffer while git blocks on a full stdout buffer.
    """
    shas = [s for s in dict.fromkeys(shas) if s]
    if not shas:
        return {}
    p = subprocess.Popen(["git", "cat-file", "--batch"], cwd=REPO,
                         stdin=subprocess.PIPE, stdout=subprocess.PIPE)

    def feed():
        try:
            p.stdin.write(("\n".join(shas) + "\n").encode())
            p.stdin.close()
        except (OSError, ValueError):
            pass

    t = threading.Thread(target=feed)
    t.daemon = True
    t.start()

    out = {}
    for _ in shas:
        header = p.stdout.readline()
        if not header:
            break
        parts = header.decode("utf-8", "replace").split()
        if len(parts) < 3:
            continue                      # `<sha> missing` carries no payload
        sha, size = parts[0], int(parts[2])
        out[sha] = p.stdout.read(size)
        p.stdout.read(1)
    p.stdout.close()
    p.wait()
    return out


def files_at(rev, suffix=".rs"):
    """path -> decoded text, for every file of one kind at a revision."""
    t = tree(rev, suffix)
    blobs = batch_blobs(t.values())
    return {p: blobs.get(sha, b"").decode("utf-8", "replace") for p, sha in t.items()}
