"""Metering relay between an agent and Z.ai.

Four agents cannot be compared by their own cost reports: each computes dollars from its own
price table, and two of them do not report dollars at all. This puts one meter in front of the
provider instead, so every agent is measured by the same code reading the provider's own `usage`.

It also normalises the two request fields that would otherwise decide the result:

  stream_options.include_usage = true   Z.ai omits `usage` entirely without it, and only the
                                        codex binaries set it, so the others would measure as zero.
  reasoning_effort = <--effort>          an absent `reasoning_effort` means `max` on GLM, so tools
                                        that send nothing would run deeper than the ones that do.

What each agent originally sent is logged before the rewrite, so the report can still say what a
tool would have done unmanaged.

Started fresh per run and killed after it; runs are sequential, so one process is one run and no
request needs tagging.
"""

import argparse
import json
import os
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import httpx

UPSTREAM = "https://api.z.ai/api/paas/v4"
HOP_BY_HOP = {
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
    "host",
    "content-length",
    "accept-encoding",
}


def upstream_url(path):
    """Map any client path onto the Z.ai base.

    Clients disagree about the suffix - the codex binaries are configured with the full
    `/api/paas/v4` base and append `/chat/completions`, while OpenAI-compatible clients append
    `/v1/chat/completions`. Both are reduced to the same upstream call.
    """
    p = path.split("?", 1)[0]
    for prefix in ("/api/paas/v4", "/v1", "/v4"):
        if p.startswith(prefix):
            p = p[len(prefix):]
            break
    if not p.startswith("/"):
        p = "/" + p
    return UPSTREAM + p


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    args = None
    client = None

    def log_message(self, *a):  # silence the default stderr access log
        pass

    def _record(self, row):
        row["label"] = self.args.label
        row["ts"] = time.strftime("%Y-%m-%dT%H:%M:%S")
        with open(self.args.out, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(row) + "\n")

    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(length) if length else b""

        sent_effort = None
        sent_usage = None
        model = None
        stream = True
        try:
            body = json.loads(raw)
        except ValueError:
            body = None

        if isinstance(body, dict):
            sent_effort = body.get("reasoning_effort")
            sent_usage = (body.get("stream_options") or {}).get("include_usage")
            model = body.get("model")
            stream = bool(body.get("stream", False))
            # The two rewrites. Everything else is forwarded untouched.
            body["stream_options"] = {"include_usage": True}
            if self.args.effort:
                # GLM gates chain-of-thought behind `thinking`, and reads depth from
                # `reasoning_effort`. codex-api sends both; the other two send neither, so both
                # have to be normalised or the depth differs between agents.
                body["thinking"] = {"type": "enabled"}
                body["reasoning_effort"] = self.args.effort
            raw = json.dumps(body).encode("utf-8")

        headers = {
            k: v for k, v in self.headers.items() if k.lower() not in HOP_BY_HOP
        }
        headers["Content-Type"] = "application/json"
        # The relay holds the credential, not the agents. They can be configured with a placeholder,
        # which is then what lands in their config files on disk - Cline writes its key to
        # `settings/providers.json` in plain text, and there is no reason for a real one to be there.
        key = os.environ.get("ZAI_API_KEY")
        if key:
            for k in [h for h in headers if h.lower() == "authorization"]:
                del headers[k]
            headers["Authorization"] = "Bearer " + key

        # Geçici teşhis: her etiketin ilk gövdesini bir kez diske yaz.
        if os.environ.get("RELAY_DUMP_BODY"):
            dump = f"{self.args.out}.{self.args.label}.body.json"
            if not os.path.exists(dump) and isinstance(body, dict) and body.get("tools"):
                with open(dump, "w", encoding="utf-8") as fh:
                    fh.write(raw.decode("utf-8", "replace"))

        started = time.time()
        usage = None
        status = 0
        try:
            with self.client.stream(
                "POST", upstream_url(self.path), content=raw, headers=headers
            ) as up:
                status = up.status_code
                self.send_response(status)
                for k, v in up.headers.items():
                    if k.lower() not in HOP_BY_HOP:
                        self.send_header(k, v)
                self.send_header("Transfer-Encoding", "chunked")
                self.end_headers()

                tail = b""
                for chunk in up.iter_raw():
                    if not chunk:
                        continue
                    # Pass through immediately: buffering would change the agent's own timing.
                    self.wfile.write(b"%x\r\n" % len(chunk) + chunk + b"\r\n")
                    self.wfile.flush()
                    tail = (tail + chunk)[-65536:]
                self.wfile.write(b"0\r\n\r\n")
                self.wfile.flush()
                usage = self._usage_from(tail, stream)
        except Exception as exc:  # noqa: BLE001 - a relay must not take the run down
            self._record(
                {
                    "error": repr(exc)[:200],
                    "model": model,
                    "status": status,
                    "sent_reasoning_effort": sent_effort,
                    "sent_include_usage": sent_usage,
                    "ms": int((time.time() - started) * 1000),
                }
            )
            return

        details = (usage or {}).get("prompt_tokens_details") or {}
        self._record(
            {
                "model": model,
                "status": status,
                "prompt_tokens": (usage or {}).get("prompt_tokens", 0),
                "cached_tokens": details.get("cached_tokens", 0),
                "completion_tokens": (usage or {}).get("completion_tokens", 0),
                "sent_reasoning_effort": sent_effort,
                "sent_include_usage": sent_usage,
                "ms": int((time.time() - started) * 1000),
            }
        )

    def _usage_from(self, tail, stream):
        """Pull `usage` out of the tail of the response.

        Streaming: the last `data:` frame carrying a usage object. Non-streaming: the whole body
        is one JSON document, so the tail parses directly.
        """
        if not stream:
            try:
                return json.loads(tail.decode("utf-8", "replace")).get("usage")
            except ValueError:
                pass
        found = None
        for line in tail.decode("utf-8", "replace").splitlines():
            line = line.strip()
            if not line.startswith("data:"):
                continue
            payload = line[5:].strip()
            if payload == "[DONE]":
                continue
            try:
                obj = json.loads(payload)
            except ValueError:
                continue
            if isinstance(obj, dict) and obj.get("usage"):
                found = obj["usage"]
        return found


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8788)
    ap.add_argument("--label", required=True, help="run label, e.g. suffice-rep1")
    ap.add_argument("--out", required=True, help="JSONL log path")
    ap.add_argument(
        "--effort",
        default="high",
        help="reasoning_effort forced on every request; empty string disables the rewrite",
    )
    args = ap.parse_args()

    os.makedirs(os.path.dirname(os.path.abspath(args.out)) or ".", exist_ok=True)
    Handler.args = args
    Handler.client = httpx.Client(timeout=httpx.Timeout(600.0, connect=30.0))

    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    print(f"relay {args.label} on 127.0.0.1:{args.port} -> {UPSTREAM}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    sys.exit(main())
