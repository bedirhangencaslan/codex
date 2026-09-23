/** Minimal JSON-RPC 2.0 client for `suffice app-server --listen ws://…`.
 *
 * The wire format matches codex-rs/app-server/README.md: one JSON-RPC message
 * per websocket text frame, `"jsonrpc":"2.0"` omitted on the wire. This client
 * only *reads* documented endpoints; it never constructs model-visible input.
 */

type Pending = {
  resolve: (v: unknown) => void;
  reject: (e: Error) => void;
};

export type NotificationHandler = (method: string, params: unknown) => void;

export class AppServerClient {
  private ws: WebSocket | null = null;
  private nextId = 1;
  private pending = new Map<number, Pending>();
  private handlers = new Set<NotificationHandler>();

  constructor(private url: string) {}

  onNotification(handler: NotificationHandler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  async connect(): Promise<void> {
    await new Promise<void>((resolve, reject) => {
      const ws = new WebSocket(this.url);
      ws.onopen = () => {
        this.ws = ws;
        resolve();
      };
      ws.onerror = () => reject(new Error(`could not connect to app-server at ${this.url}`));
      ws.onclose = () => {
        this.ws = null;
        const err = new Error("app-server connection closed");
        for (const p of this.pending.values()) p.reject(err);
        this.pending.clear();
      };
      ws.onmessage = (ev) => this.handleMessage(String(ev.data));
    });
    await this.request("initialize", {
      clientInfo: { name: "suffice-gui", title: "Suffice GUI", version: "0.1.0" },
    });
    this.notify("initialized", {});
  }

  close(): void {
    this.ws?.close();
  }

  request<T = unknown>(method: string, params: unknown): Promise<T> {
    const ws = this.ws;
    if (!ws) return Promise.reject(new Error("not connected"));
    const id = this.nextId++;
    const promise = new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
    });
    ws.send(JSON.stringify({ id, method, params }));
    return promise;
  }

  notify(method: string, params: unknown): void {
    this.ws?.send(JSON.stringify({ method, params }));
  }

  private handleMessage(raw: string): void {
    let msg: { id?: number; method?: string; params?: unknown; result?: unknown; error?: { message?: string } };
    try {
      msg = JSON.parse(raw);
    } catch {
      return;
    }
    if (msg.id != null && msg.method == null) {
      const p = this.pending.get(msg.id);
      if (!p) return;
      this.pending.delete(msg.id);
      if (msg.error) p.reject(new Error(msg.error.message ?? "app-server error"));
      else p.resolve(msg.result);
      return;
    }
    if (msg.method != null && msg.id == null) {
      for (const h of this.handlers) h(msg.method, msg.params);
    }
    // Server→client *requests* (approvals etc.) are out of scope for the
    // read-only screens in Phase 1 and intentionally not answered here.
  }
}
