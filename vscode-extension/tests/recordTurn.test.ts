// Records the notifications of one real turn into tests/fixtures/turn-notifications.json, so the
// chat reducer is tested against what the server actually sends. Opt-in and paid (one small GLM
// request): RECORD_TURN=1 SUFFICE_BIN=<suffice.exe> ZAI_API_KEY=<key> npx jest tests/recordTurn.test.ts
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";
import { JsonRpcConnection } from "../src/protocol/jsonrpc";
import { AppServerSession } from "../src/protocol/session";
import { StdioTransport } from "../src/protocol/stdioTransport";

const enabled = process.env.RECORD_TURN && process.env.SUFFICE_BIN && process.env.ZAI_API_KEY;
const maybe = enabled ? test : test.skip;

maybe(
  "record one real turn",
  async () => {
    const home = mkdtempSync(join(tmpdir(), "suffice-record-"));
    writeFileSync(
      join(home, "config.toml"),
      ['model = "glm-5.3-flash"', 'model_provider = "zai"', 'approval_policy = "never"', ""].join("\n"),
    );
    const transport = new StdioTransport({
      binary: process.env.SUFFICE_BIN!,
      cwd: home,
      env: { ...process.env, SUFFICE_HOME: home },
    });
    const rpc = new JsonRpcConnection(transport);
    const session = new AppServerSession(rpc);
    const seen: Array<{ method: string; params: unknown }> = [];
    let done: () => void = () => {};
    const completed = new Promise<void>((resolve) => (done = resolve));
    rpc.onNotification((n) => {
      seen.push(n);
      if (n.method === "turn/completed") done();
    });
    try {
      await session.initialize({ name: "suffice-vscode-record", title: null, version: "0.0.0" });
      const started = await session.threadStart({ cwd: home, sandbox: "danger-full-access" });
      await session.turnStart({
        threadId: started.thread.id,
        input: [{ type: "text", text: "Run the shell command `echo hello-suffice` and then reply with one short sentence saying what it printed.", text_elements: [] }],
        invisible: false,
      });
      await completed;
    } finally {
      rpc.close();
    }
    const scrub = (text: string) =>
      text.split(home).join("<home>").split(home.replace(/\\/g, "\\\\")).join("<home>").split(homedir()).join("<user>");
    writeFileSync(
      join(__dirname, "fixtures", "turn-notifications.json"),
      scrub(JSON.stringify(seen, null, 2)) + "\n",
    );
    try {
      rmSync(home, { recursive: true, force: true });
    } catch {
      // Windows may still hold a file for a moment
    }
    expect(seen.some((n) => n.method === "turn/completed")).toBe(true);
  },
  180_000,
);
