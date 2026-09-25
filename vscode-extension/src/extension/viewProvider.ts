// The side-bar webview (a WebviewView, like Claude Code's panel, not an editor tab). It serves
// the bundled React app under a strict CSP and routes messages between it, the SessionHost and
// the extension's storage.
import { randomBytes } from "node:crypto";
import * as vscode from "vscode";
import { resolveLocale } from "../shared/i18n";
import { ALLOWED_RPC_METHODS, type HostCommand, type HostToWebview, type InitState, type WebviewToHost } from "../shared/messages";
import type { SessionHost } from "./sessionHost";
import type { ExtensionStorage } from "./storage";

export class SufficeViewProvider implements vscode.WebviewViewProvider {
  static readonly viewId = "suffice.chat";
  private view: vscode.WebviewView | undefined;
  private readonly queued: HostToWebview[] = [];

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly host: SessionHost,
    private readonly storage: ExtensionStorage,
    private readonly restartServer: () => Promise<void>,
  ) {}

  resolveWebviewView(view: vscode.WebviewView): void {
    this.view = view;
    const media = vscode.Uri.joinPath(this.context.extensionUri, "dist");
    view.webview.options = { enableScripts: true, localResourceRoots: [media] };
    view.webview.html = this.html(view.webview, media);
    view.webview.onDidReceiveMessage((message: WebviewToHost) => void this.onMessage(message));
    view.onDidDispose(() => {
      this.view = undefined;
    });
  }

  post(message: HostToWebview): void {
    if (this.view) void this.view.webview.postMessage(message);
    else if (message.type === "command") this.queued.push(message);
  }

  sendCommand(command: HostCommand): void {
    this.post({ type: "command", command });
  }

  async pushState(patch: Partial<InitState>): Promise<void> {
    this.post({ type: "stateChanged", patch });
  }

  async initState(): Promise<InitState> {
    const config = vscode.workspace.getConfiguration("suffice");
    const languageSetting = config.get<string>("language", "auto");
    return {
      ...this.storage.read(),
      locale: resolveLocale(languageSetting, vscode.env.language),
      languageSetting,
      themeId: config.get<string>("theme", "vscode"),
      workspaceFolders: (vscode.workspace.workspaceFolders ?? []).map((f) => ({ name: f.name, path: f.uri.fsPath })),
      envKeys: await this.storage.envKeys(),
      extensionVersion: String(this.context.extension.packageJSON.version ?? "0.0.0"),
    };
  }

  private async onMessage(message: WebviewToHost): Promise<void> {
    switch (message.type) {
      case "ready":
        this.post({ type: "init", state: await this.initState() });
        this.post({ type: "server", status: this.host.currentStatus });
        this.host.replayPending();
        for (const queued of this.queued.splice(0)) this.post(queued);
        return;
      case "rpc": {
        if (!ALLOWED_RPC_METHODS.has(message.method)) {
          this.post({ type: "rpcResult", id: message.id, error: { message: `method not allowed: ${message.method}` } });
          return;
        }
        try {
          const result = await this.host.request(message.method, message.params);
          this.post({ type: "rpcResult", id: message.id, result });
        } catch (error) {
          const e = error as { message?: string; code?: number };
          this.post({ type: "rpcResult", id: message.id, error: { message: e.message ?? String(error), code: e.code } });
        }
        return;
      }
      case "serverRequestResult":
        this.host.answer(message.requestId, message.result);
        return;
      case "setSetting": {
        const key = message.key === "language" ? "language" : "theme";
        await vscode.workspace.getConfiguration("suffice").update(key, message.value, vscode.ConfigurationTarget.Global);
        return;
      }
      case "setState":
        await this.storage.write(message.key, message.value);
        return;
      case "setSecret":
        await this.storage.setSecret(message.name, message.value);
        await this.pushState({ envKeys: await this.storage.envKeys() });
        await this.restartServer();
        return;
      case "restartServer":
        await this.restartServer();
        return;
      case "openSettings":
        await vscode.commands.executeCommand("workbench.action.openSettings", "suffice.");
        return;
      case "openFile": {
        const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(message.path));
        await vscode.window.showTextDocument(doc, { preview: true });
        return;
      }
      case "copy":
        await vscode.env.clipboard.writeText(message.text);
        return;
    }
  }

  private html(webview: vscode.Webview, media: vscode.Uri): string {
    const nonce = randomBytes(16).toString("base64");
    const script = webview.asWebviewUri(vscode.Uri.joinPath(media, "webview.js"));
    const style = webview.asWebviewUri(vscode.Uri.joinPath(media, "webview.css"));
    const csp = [
      "default-src 'none'",
      `img-src ${webview.cspSource} data:`,
      `font-src ${webview.cspSource}`,
      `style-src ${webview.cspSource} 'unsafe-inline'`,
      `script-src 'nonce-${nonce}'`,
    ].join("; ");
    return `<!doctype html>
<html>
<head>
<meta charset="utf-8" />
<meta http-equiv="Content-Security-Policy" content="${csp}" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<link rel="stylesheet" href="${style}" />
</head>
<body>
<div id="root"></div>
<script nonce="${nonce}" src="${script}"></script>
</body>
</html>`;
  }
}
