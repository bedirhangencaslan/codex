#!/usr/bin/env python3
"""Ağ sızıntı gözlemcisi: bir komutu ÇEVRELEYEREK (kodunu değiştirmeden) tüm
dışa bağlantılarını kaydeder.

İki katman:
  1. CONNECT proxy — komuta HTTPS_PROXY/HTTP_PROXY olarak verilir; proxy'yi
     onurlandıran her istemcinin hedef alan adı, portu ve bayt sayıları loglanır.
     TLS İÇERİĞİ AÇILMAZ: bu bir meta veri gözlemidir ("kiminle konuşuyor").
  2. lsof anketi — süreç ağacının doğrudan (proxy'siz) açtığı soketler
     ~0,5 sn'de bir taranır; proxy'yi bypass eden her bağlantı görünür.

Kanarya: hedef alan adlarında verilen token aranır (DNS/hostname üzerinden
sızdırma tespiti). Rapor JSON'dur; allowlist dışındaki her temas 'unexpected'.

Kullanım:
  netwatch.py --allow api.z.ai --canary TOKEN [--timeout 900] -- CMD ARG...
"""

import argparse
import json
import select
import socket
import subprocess
import sys
import threading
import time

events = []
events_lock = threading.Lock()


def log_event(kind, **kw):
    with events_lock:
        events.append({"ts": round(time.time(), 2), "kind": kind, **kw})


def tunnel(client, host, port):
    try:
        remote = socket.create_connection((host, port), timeout=20)
    except OSError as e:
        client.sendall(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")
        client.close()
        log_event("proxy_connect_failed", host=host, port=port, error=str(e))
        return
    client.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
    up = down = 0
    socks = [client, remote]
    try:
        while True:
            r, _, _ = select.select(socks, [], [], 60)
            if not r:
                break
            done = False
            for s in r:
                data = s.recv(65536)
                if not data:
                    done = True
                    break
                if s is client:
                    remote.sendall(data)
                    up += len(data)
                else:
                    client.sendall(data)
                    down += len(data)
            if done:
                break
    except OSError:
        pass
    finally:
        client.close()
        remote.close()
        log_event("proxy_tunnel", host=host, port=port, bytes_out=up, bytes_in=down)


def handle(client):
    try:
        client.settimeout(15)
        head = b""
        while b"\r\n\r\n" not in head and len(head) < 65536:
            chunk = client.recv(4096)
            if not chunk:
                client.close()
                return
            head += chunk
        line = head.split(b"\r\n", 1)[0].decode("latin1")
        parts = line.split()
        if len(parts) >= 2 and parts[0] == "CONNECT":
            host, _, port = parts[1].partition(":")
            log_event("proxy_connect", host=host, port=int(port or 443))
            client.settimeout(None)
            tunnel(client, host, int(port or 443))
        else:
            # düz HTTP proxy isteği (nadir) — hedefi logla, reddet
            target = parts[1] if len(parts) > 1 else "?"
            log_event("proxy_http", target=target)
            client.sendall(b"HTTP/1.1 501 Not Implemented\r\n\r\n")
            client.close()
    except OSError:
        client.close()


def proxy_server(sock):
    while True:
        try:
            c, _ = sock.accept()
        except OSError:
            return
        threading.Thread(target=handle, args=(c,), daemon=True).start()


def descendants(root_pid):
    try:
        out = subprocess.run(["ps", "-axo", "pid=,ppid="], capture_output=True, text=True, timeout=5).stdout
    except Exception:
        return {root_pid}
    kids = {}
    for ln in out.splitlines():
        try:
            pid, ppid = map(int, ln.split())
        except ValueError:
            continue
        kids.setdefault(ppid, []).append(pid)
    seen, stack = {root_pid}, [root_pid]
    while stack:
        for k in kids.get(stack.pop(), []):
            if k not in seen:
                seen.add(k)
                stack.append(k)
    return seen


def lsof_poll(root_pid, stop, proxy_port):
    seen = set()
    while not stop.is_set():
        pids = descendants(root_pid)
        try:
            out = subprocess.run(
                ["lsof", "-nP", "-a", "-i", "TCP", "-p", ",".join(map(str, sorted(pids)))],
                capture_output=True, text=True, timeout=10,
            ).stdout
        except Exception:
            out = ""
        for ln in out.splitlines():
            if "->" not in ln:
                continue
            remote = ln.split("->")[-1].split()[0]
            ip, _, port = remote.rpartition(":")
            key = (ip, port)
            if key in seen:
                continue
            seen.add(key)
            if ip in ("127.0.0.1", "::1", "[::1]"):
                if port != str(proxy_port):
                    log_event("direct_local", addr=remote)
                continue
            try:
                name = socket.getnameinfo((ip, int(port)), 0)[0]
            except OSError:
                name = ip
            log_event("direct_remote", ip=ip, port=port, rdns=name)
        stop.wait(0.5)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--allow", action="append", default=[], help="beklenen alan adı (soneki eşleşir)")
    ap.add_argument("--canary", default=None)
    ap.add_argument("--timeout", type=int, default=900)
    ap.add_argument("--report", default=None, help="JSON rapor dosyası (varsayılan stdout)")
    ap.add_argument("cmd", nargs=argparse.REMAINDER)
    args = ap.parse_args()
    cmd = args.cmd[1:] if args.cmd and args.cmd[0] == "--" else args.cmd
    if not cmd:
        ap.error("komut verin: netwatch.py [opsiyonlar] -- CMD ...")

    sock = socket.socket()
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("127.0.0.1", 0))
    sock.listen(64)
    port = sock.getsockname()[1]
    threading.Thread(target=proxy_server, args=(sock,), daemon=True).start()

    import os
    env = os.environ.copy()
    proxy = f"http://127.0.0.1:{port}"
    env.update({"HTTPS_PROXY": proxy, "HTTP_PROXY": proxy, "https_proxy": proxy, "http_proxy": proxy,
                "ALL_PROXY": proxy, "NO_PROXY": "", "no_proxy": ""})

    t0 = time.time()
    proc = subprocess.Popen(cmd, env=env)
    stop = threading.Event()
    poller = threading.Thread(target=lsof_poll, args=(proc.pid, stop, port), daemon=True)
    poller.start()
    try:
        rc = proc.wait(timeout=args.timeout)
    except subprocess.TimeoutExpired:
        proc.kill()
        rc = -9
    time.sleep(1.0)
    stop.set()
    poller.join(timeout=3)
    sock.close()

    hosts = sorted({e["host"] for e in events if e["kind"] == "proxy_connect"})
    direct = sorted({f'{e["rdns"]}:{e["port"]}' for e in events if e["kind"] == "direct_remote"})

    def allowed(name):
        return any(name == a or name.endswith("." + a) for a in args.allow)

    unexpected = [h for h in hosts if not allowed(h)]
    unexpected += [d for d in direct if not allowed(d.rsplit(":", 1)[0].rstrip("."))]
    canary_hits = []
    if args.canary:
        blob = json.dumps(events)
        if args.canary in blob:
            canary_hits = [e for e in events if args.canary in json.dumps(e)]

    report = {
        "cmd": cmd,
        "exit_code": rc,
        "duration_s": round(time.time() - t0, 1),
        "proxied_hosts": hosts,
        "direct_remote": direct,
        "allow": args.allow,
        "unexpected": unexpected,
        "canary_leak": canary_hits,
        "verdict": "LEAK-SUSPECT" if (unexpected or canary_hits) else "CLEAN",
        "events": events,
    }
    out = json.dumps(report, ensure_ascii=False, indent=2)
    if args.report:
        with open(args.report, "w") as f:
            f.write(out)
        print(f"verdict={report['verdict']} hosts={hosts} direct={direct} → {args.report}")
    else:
        print(out)


if __name__ == "__main__":
    main()
