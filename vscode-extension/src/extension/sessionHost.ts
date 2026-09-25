// Owns the `suffice app-server` process for the extension: spawns it with the user's stored
// API keys in its environment, performs the initialize handshake, relays notifications to the
// webview and routes the server's requests (approvals, questions) to the webview and back.
import { JsonRpcConnection, type IncomingServerRequest } from "../protocol/jsonrpc";
import { AppServerSession } from "../protocol/session";
import { StdioTransport } from "../protocol/stdioTransport";
import type { ServerStatus } from "../shared/messages";

export interface SessionHostEvents {
  status(status: ServerStatus): void;
  notification(method: string, params: unknown): void;
  serverRequest(requestId: number, method: string, params: unknown): void;
  log(text: string): void;
}

export interface StartOptions {
  binary: string | undefined;
  cwd: string;
  env: NodeJS.ProcessEnv;
  version: string;
}

interface PendingServerRequest {
  method: string;
  params: unknown;
  resolve: (result: unknown) => void;
}

export class SessionHost {
  private rpc: JsonRpcConnection | undefined;
  private session: AppServerSession | undefined;
  private status: ServerStatus = { state: "stopped", reason: "not started" };
  private nextServerRequestId = 1;
  private readonly pending = new Map<number, PendingServerRequest>();
  private generation = 0;

  constructor(private readonly events: SessionHostEvents) {}

  get currentStatus(): ServerStatus {
    return this.status;
  }

  async start(options: StartOptions): Promise<void> {
    this.stop();
    const generation = ++this.generation;
    if (!options.binary) {
      this.setStatus({ state: "noBinary" });
      return;
    }
    this.setStatus({ state: "starting" });
    this.events.log(`starting ${options.binary} app-server in ${options.cwd}`);

    const transport = new StdioTransport({
      binary: options.binary,
      cwd: options.cwd,
      env: options.env,
      onStderr: (text) => this.events.log(text.trimEnd()),
    });
    const rpc = new JsonRpcConnection(transport, (m) => this.events.log(m));
    this.rpc = rpc;
    this.session = new AppServerSession(rpc);

    transport.onClose((reason) => {
      if (generation !== this.generation) return;
      this.events.log(reason);
      this.failPending();
      this.setStatus({ state: "stopped", reason });
    });
    rpc.onNotification((n) => this.events.notification(n.method, n.params));
    rpc.setServerRequestHandler((request) => this.forwardServerRequest(request));

    try {
      const init = await this.session.initialize({
        name: "suffice_vscode",
        title: "Suffice for VS Code",
        version: options.version,
      });
      if (generation !== this.generation) return;
      this.setStatus({ state: "ready", codexHome: init.codexHome, userAgent: init.userAgent, cwd: options.cwd });
    } catch (error) {
      if (generation !== this.generation) return;
      const reason = error instanceof Error ? error.message : String(error);
      this.events.log(`initialize failed: ${reason}`);
      this.setStatus({ state: "stopped", reason });
    }
  }

  stop(): void {
    this.generation++;
    this.failPending();
    this.rpc?.close();
    this.rpc = undefined;
    this.session = undefined;
  }

  /** Forwards one allow-listed protocol call from the webview. */
  request(method: string, params: unknown): Promise<unknown> {
    if (!this.rpc || this.status.state !== "ready") {
      return Promise.reject(new Error("Suffice is not running"));
    }
    return this.rpc.request(method, params);
  }

  /** The webview answered a server request. */
  answer(requestId: number, result: unknown): void {
    const pending = this.pending.get(requestId);
    if (!pending) return;
    this.pending.delete(requestId);
    pending.resolve(result);
  }

  /** Re-sends requests still waiting for an answer, e.g. after the webview reloaded. */
  replayPending(): void {
    for (const [id, p] of this.pending) this.events.serverRequest(id, p.method, p.params);
  }

  private forwardServerRequest(request: IncomingServerRequest): Promise<unknown> {
    const id = this.nextServerRequestId++;
    return new Promise((resolve) => {
      this.pending.set(id, { method: request.method, params: request.params, resolve });
      this.events.serverRequest(id, request.method, request.params);
    });
  }

  private failPending(): void {
    // Declining keeps the server consistent if it is still alive; a dead server ignores it.
    for (const [, p] of this.pending) p.resolve(declineFor(p.method));
    this.pending.clear();
  }

  private setStatus(status: ServerStatus): void {
    this.status = status;
    this.events.status(status);
  }
}

export function declineFor(method: string): unknown {
  switch (method) {
    case "item/commandExecution/requestApproval":
    case "item/fileChange/requestApproval":
      return { decision: "cancel" };
    case "item/tool/requestUserInput":
      return { answers: {} };
    default:
      return {};
  }
}
