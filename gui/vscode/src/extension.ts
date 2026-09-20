// Suffice GUI as a VS Code webview. Like the Tauri shell this is a dumb
// frame: it serves the exact gui/web bundle (synced into media/ at build
// time) and rewrites asset URLs for the webview origin. All data flows over
// the app-server WebSocket; the extension adds no model-visible surface.
import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("suffice.openGui", () => {
      const mediaRoot = vscode.Uri.file(path.join(context.extensionPath, "media"));
      const indexPath = path.join(context.extensionPath, "media", "index.html");
      if (!fs.existsSync(indexPath)) {
        void vscode.window.showErrorMessage(
          "Suffice GUI paketi bulunamadı — önce `pnpm --filter suffice-gui build` çalıştırın.",
        );
        return;
      }

      const panel = vscode.window.createWebviewPanel("sufficeGui", "Suffice", vscode.ViewColumn.One, {
        enableScripts: true,
        localResourceRoots: [mediaRoot],
        retainContextWhenHidden: true,
      });

      const assetBase = panel.webview.asWebviewUri(mediaRoot).toString();
      // vite is configured with base "./"; point those relative refs at the
      // webview resource origin instead.
      const html = fs
        .readFileSync(indexPath, "utf8")
        .replaceAll('src="./', `src="${assetBase}/`)
        .replaceAll('href="./', `href="${assetBase}/`);
      panel.webview.html = html;
    }),
  );
}

export function deactivate(): void {}
