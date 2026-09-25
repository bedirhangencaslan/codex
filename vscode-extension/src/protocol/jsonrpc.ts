// JSON-RPC connection for `suffice app-server`, independent of the transport.
//
// Wire format (codex-rs/app-server-protocol/src/rpc.rs): one JSON object per line and no
// `"jsonrpc": "2.0"` field. Requests are `{id, method, params}`, notifications `{method, params}`,
// responses `{id, result}` or `{id, error: {code, message, data?}}`. The server also sends
// requests of its own (approvals, user-input questions); they are answered with a response
// carrying the server's id. This mirrors the Python SDK's `CodexClient` (sdk/python/src/
// openai_codex/client.py), which is the reference client for this protocol.

export interface Transport {
  send(line: string): void;
  onLine(listener: (line: string) => void): void;
  onClose(listener: (reason: string) => void): void;
  close(): void;
}

export interface RpcErrorBody {
  code: number;
  message: string;
  data?: unknown;
}

export class JsonRpcError extends Error {
  readonly code: number;
  readonly data: unknown;
  readonly method: string;

  constructor(method: string, body: RpcErrorBody) {
    super(`${method}: ${body.message}`);
    this.name = "JsonRpcError";
    this.code = body.code;
    this.data = body.data;
    this.method = method;
  }
}

export class ConnectionClosedError extends Error {
  constructor(reason: string) {
    super(`app-server connection closed: ${reason}`);
    this.name = "ConnectionClosedError";
  }
}

export type RequestId = number | string;

export interface IncomingNotification {
  method: string;
  params: unknown;
}

export interface IncomingServerRequest {
  id: RequestId;
  method: string;
  params: unknown;
}

type ServerRequestHandler = (request: IncomingServerRequest) => Promise<unknown> | unknown;

interface Pending {
  method: string;
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

export class JsonRpcConnection {
  private nextId = 1;
  private readonly pending = new Map<RequestId, Pending>();
  private readonly notificationListeners = new Set<(n: IncomingNotification) => void>();
  private serverRequestHandler: ServerRequestHandler | undefined;
  private closedReason: string | undefined;

  constructor(
    private readonly transport: Transport,
    private readonly log: (message: string) => void = () => {},
  ) {
    transport.onLine((line) => this.handleLine(line));
    transport.onClose((reason) => this.handleClose(reason));
  }

  get closed(): boolean {
    return this.closedReason !== undefined;
  }

  request<T>(method: string, params?: unknown): Promise<T> {
    if (this.closedReason !== undefined) {
      return Promise.reject(new ConnectionClosedError(this.closedReason));
    }
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { method, resolve: resolve as (v: unknown) => void, reject });
      this.write(params === undefined ? { id, method } : { id, method, params });
    });
  }

  notify(method: string, params?: unknown): void {
    if (this.closedReason !== undefined) return;
    this.write(params === undefined ? { method } : { method, params });
  }

  onNotification(listener: (notification: IncomingNotification) => void): () => void {
    this.notificationListeners.add(listener);
    return () => this.notificationListeners.delete(listener);
  }

  /** One handler answers every server-initiated request (approvals, questions, ...). */
  setServerRequestHandler(handler: ServerRequestHandler): void {
    this.serverRequestHandler = handler;
  }

  close(): void {
    this.transport.close();
    this.handleClose("closed by client");
  }

  private write(message: object): void {
    this.transport.send(JSON.stringify(message));
  }

  private handleLine(line: string): void {
    const text = line.trim();
    if (!text) return;
    let message: Record<string, unknown>;
    try {
      message = JSON.parse(text) as Record<string, unknown>;
    } catch {
      this.log(`app-server sent a non-JSON line: ${text.slice(0, 200)}`);
      return;
    }
    const hasId = "id" in message && message.id !== null && message.id !== undefined;
    const method = typeof message.method === "string" ? message.method : undefined;

    if (hasId && method === undefined) {
      this.handleResponse(message.id as RequestId, message);
    } else if (hasId && method !== undefined) {
      void this.handleServerRequest({ id: message.id as RequestId, method, params: message.params });
    } else if (method !== undefined) {
      for (const listener of this.notificationListeners) {
        listener({ method, params: message.params });
      }
    } else {
      this.log(`app-server sent an unrecognised message: ${text.slice(0, 200)}`);
    }
  }

  private handleResponse(id: RequestId, message: Record<string, unknown>): void {
    const pending = this.pending.get(id);
    if (!pending) {
      this.log(`response for unknown request id ${String(id)}`);
      return;
    }
    this.pending.delete(id);
    if (message.error && typeof message.error === "object") {
      pending.reject(new JsonRpcError(pending.method, message.error as RpcErrorBody));
    } else {
      pending.resolve(message.result);
    }
  }

  private async handleServerRequest(request: IncomingServerRequest): Promise<void> {
    const handler = this.serverRequestHandler;
    if (!handler) {
      this.write({
        id: request.id,
        error: { code: -32601, message: `client does not handle ${request.method}` },
      });
      return;
    }
    try {
      const result = await handler(request);
      this.write({ id: request.id, result: result ?? {} });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.write({ id: request.id, error: { code: -32603, message } });
    }
  }

  private handleClose(reason: string): void {
    if (this.closedReason !== undefined) return;
    this.closedReason = reason;
    const error = new ConnectionClosedError(reason);
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
  }
}
