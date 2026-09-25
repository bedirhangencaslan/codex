// Extension entry point: one SessionHost (the app-server process), one side-bar view.
import { homedir } from "node:os";
import * as vscode from "vscode";
import { resolveLocale } from "../shared/i18n";
import { findSufficeBinary } from "./binary";
import { SessionHost } from "./sessionHost";
import { ExtensionStorage } from "./storage";
import { SufficeViewProvider } from "./viewProvider";

export function activate(context: vscode.ExtensionContext): void {
  const output = vscode.window.createOutputChannel("Suffice");
  const storage = new ExtensionStorage(context);
  let provider: SufficeViewProvider | undefined;

  const host = new SessionHost({
    status: (status) => provider?.post({ type: "server", status }),
    notification: (method, params) => provider?.post({ type: "notification", method, params }),
    serverRequest: (requestId, method, params) => provider?.post({ type: "serverRequest", requestId, method, params }),
    log: (text) => output.appendLine(text),
  });

  const start = async () => {
    const folders = (vscode.workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath);
    const binary = findSufficeBinary(vscode.workspace.getConfiguration("suffice").get<string>("binaryPath"), folders);
    await host.start({
      binary,
      cwd: folders[0] ?? homedir(),
      env: { ...process.env, ...(await storage.envOverrides()) },
      version: String(context.extension.packageJSON.version ?? "0.0.0"),
    });
  };

  provider = new SufficeViewProvider(context, host, storage, start);
  context.subscriptions.push(
    output,
    vscode.window.registerWebviewViewProvider(SufficeViewProvider.viewId, provider, {
      webviewOptions: { retainContextWhenHidden: true },
    }),
    vscode.commands.registerCommand("suffice.focus", () => vscode.commands.executeCommand("suffice.chat.focus")),
    vscode.commands.registerCommand("suffice.newThread", async () => {
      await vscode.commands.executeCommand("suffice.chat.focus");
      provider?.sendCommand("newThread");
    }),
    vscode.commands.registerCommand("suffice.showInterfaces", async () => {
      await vscode.commands.executeCommand("suffice.chat.focus");
      provider?.sendCommand("showInterfaces");
    }),
    vscode.commands.registerCommand("suffice.restartServer", start),
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration("suffice.binaryPath")) await start();
      if (e.affectsConfiguration("suffice.language") || e.affectsConfiguration("suffice.theme")) {
        const config = vscode.workspace.getConfiguration("suffice");
        const languageSetting = config.get<string>("language", "auto");
        await provider?.pushState({
          languageSetting,
          locale: resolveLocale(languageSetting, vscode.env.language),
          themeId: config.get<string>("theme", "vscode"),
        });
      }
    }),
    { dispose: () => host.stop() },
  );

  void start();
}

export function deactivate(): void {}
