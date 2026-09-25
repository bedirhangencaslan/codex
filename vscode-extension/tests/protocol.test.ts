import { ConnectionClosedError, JsonRpcConnection, JsonRpcError, type Transport } from "../src/protocol/jsonrpc";
import { LineSplitter } from "../src/protocol/stdioTransport";

class FakeTransport implements Transport {
  sent: Array<Record<string, unknown>> = [];
  private lineListener: (line: string) => void = () => {};
  private closeListener: (reason: string) => void = () => {};
  send(line: string) {
    this.sent.push(JSON.parse(line) as Record<string, unknown>);
  }
  onLine(listener: (line: string) => void) {
    this.lineListener = listener;
  }
  onClose(listener: (reason: string) => void) {
    this.closeListener = listener;
  }
  close() {}
  serverSays(message: object) {
    this.lineListener(JSON.stringify(message));
  }
  serverDies(reason: string) {
    this.closeListener(reason);
  }
}

const tick = () => new Promise((resolve) => setImmediate(resolve));

describe("JsonRpcConnection", () => {
  test("requests carry no jsonrpc field and resolve by id", async () => {
    const t = new FakeTransport();
    const rpc = new JsonRpcConnection(t);
    const first = rpc.request<{ ok: number }>("model/list", { limit: 5 });
    const second = rpc.request<{ ok: number }>("skills/list", {});
    expect(t.sent[0]).toEqual({ id: 1, method: "model/list", params: { limit: 5 } });
    expect("jsonrpc" in t.sent[0]!).toBe(false);
    t.serverSays({ id: 2, result: { ok: 2 } });
    t.serverSays({ id: 1, result: { ok: 1 } });
    await expect(first).resolves.toEqual({ ok: 1 });
    await expect(second).resolves.toEqual({ ok: 2 });
  });

  test("error responses reject with the method name", async () => {
    const t = new FakeTransport();
    const rpc = new JsonRpcConnection(t);
    const pending = rpc.request("thread/start", {});
    t.serverSays({ id: 1, error: { code: -32600, message: "Not initialized" } });
    await expect(pending).rejects.toBeInstanceOf(JsonRpcError);
    await expect(pending).rejects.toThrow("thread/start: Not initialized");
  });

  test("notifications reach listeners, notify sends no id", () => {
    const t = new FakeTransport();
    const rpc = new JsonRpcConnection(t);
    const seen: string[] = [];
    rpc.onNotification((n) => seen.push(n.method));
    t.serverSays({ method: "turn/started", params: { threadId: "t" } });
    rpc.notify("initialized");
    expect(seen).toEqual(["turn/started"]);
    expect(t.sent[0]).toEqual({ method: "initialized" });
  });

  test("server requests are answered with the server's id", async () => {
    const t = new FakeTransport();
    const rpc = new JsonRpcConnection(t);
    rpc.setServerRequestHandler(async (req) => ({ decision: req.method.includes("fileChange") ? "decline" : "accept" }));
    t.serverSays({ id: "srv-7", method: "item/fileChange/requestApproval", params: {} });
    await tick();
    expect(t.sent).toEqual([{ id: "srv-7", result: { decision: "decline" } }]);
  });

  test("an unhandled server request gets an error, not silence", async () => {
    const t = new FakeTransport();
    new JsonRpcConnection(t);
    t.serverSays({ id: 9, method: "item/tool/call", params: {} });
    await tick();
    expect(t.sent[0]).toMatchObject({ id: 9, error: { code: -32601 } });
  });

  test("closing rejects every pending request", async () => {
    const t = new FakeTransport();
    const rpc = new JsonRpcConnection(t);
    const pending = rpc.request("thread/list", {});
    t.serverDies("app-server exited with code 1");
    await expect(pending).rejects.toBeInstanceOf(ConnectionClosedError);
    await expect(rpc.request("model/list")).rejects.toBeInstanceOf(ConnectionClosedError);
  });

  test("garbage lines are logged and ignored", () => {
    const t = new FakeTransport();
    const logs: string[] = [];
    new JsonRpcConnection(t, (m) => logs.push(m));
    (t as unknown as { lineListener: (l: string) => void }).lineListener("not json");
    expect(logs[0]).toContain("non-JSON");
  });
});

describe("LineSplitter", () => {
  test("joins chunks split mid-line and strips CR", () => {
    const s = new LineSplitter();
    const out: string[] = [];
    s.push('{"a":', (l) => out.push(l));
    s.push('1}\r\n{"b":2}\n{"c"', (l) => out.push(l));
    s.push(":3}", (l) => out.push(l));
    expect(out).toEqual(['{"a":1}', '{"b":2}']);
    s.flush((l) => out.push(l));
    expect(out).toEqual(['{"a":1}', '{"b":2}', '{"c":3}']);
  });
});
