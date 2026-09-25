// The webview's side of the postMessage contract. It implements the same RpcChannel the
// extension host gives AppServerSession, so the webview uses the identical typed calls.
// Outside VS Code (a plain browser, used for previews and screenshots) a preview bridge with
// sample data stands in for the host.
import type { IncomingNotification } from "../protocol/jsonrpc";
import type { RpcChannel } from "../protocol/session";
import type { HostToWebview, WebviewToHost } from "../shared/messages";

interface VsCodeApi {
  postMessage(message: unknown): void;
  getState(): unknown;
  setState(state: unknown): void;
}

declare const acquireVsCodeApi: (() => VsCodeApi) | undefined;

export interface HostBridge extends RpcChannel {
  post(message: WebviewToHost): void;
  onMessage(listener: (message: HostToWebview) => void): () => void;
  /** Small per-view UI state that survives the view being hidden (composer history, ...). */
  viewState<T>(): T | undefined;
  setViewState<T>(state: T): void;
  readonly isPreview: boolean;
}

export class RpcError extends Error {
  constructor(
    message: string,
    readonly code?: number,
  ) {
    super(message);
  }
}

export abstract class BaseBridge implements HostBridge {
  private nextId = 1;
  private readonly pending = new Map<number, { resolve: (v: unknown) => void; reject: (e: Error) => void }>();
  private readonly listeners = new Set<(m: HostToWebview) => void>();
  abstract readonly isPreview: boolean;

  protected receive(message: HostToWebview): void {
    if (message.type === "rpcResult") {
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      if (message.error) pending.reject(new RpcError(message.error.message, message.error.code));
      else pending.resolve(message.result);
      return;
    }
    for (const listener of this.listeners) listener(message);
  }

  abstract post(message: WebviewToHost): void;
  abstract viewState<T>(): T | undefined;
  abstract setViewState<T>(state: T): void;

  onMessage(listener: (message: HostToWebview) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  request<T>(method: string, params?: unknown): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.post({ type: "rpc", id, method, params });
    });
  }

  notify(): void {
    // The webview never sends protocol notifications; the host owns `initialized`.
  }

  onNotification(listener: (notification: IncomingNotification) => void): () => void {
    return this.onMessage((m) => {
      if (m.type === "notification") listener({ method: m.method, params: m.params });
    });
  }
}

class VsCodeBridge extends BaseBridge {
  readonly isPreview = false;
  constructor(private readonly api: VsCodeApi) {
    super();
    window.addEventListener("message", (event: MessageEvent<HostToWebview>) => this.receive(event.data));
  }
  post(message: WebviewToHost): void {
    this.api.postMessage(message);
  }
  viewState<T>(): T | undefined {
    return this.api.getState() as T | undefined;
  }
  setViewState<T>(state: T): void {
    this.api.setState(state);
  }
}

export async function createBridge(): Promise<HostBridge> {
  if (typeof acquireVsCodeApi === "function") return new VsCodeBridge(acquireVsCodeApi());
  const { PreviewBridge } = await import("./preview");
  return new PreviewBridge();
}
