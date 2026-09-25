// Talks to a real `suffice app-server`. Runs only when SUFFICE_BIN points at a binary; uses a
// throwaway SUFFICE_HOME so the user's config and sessions are never read or written, and makes
// no model request (nothing here costs money).
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { JsonRpcConnection } from "../src/protocol/jsonrpc";
import { AppServerSession } from "../src/protocol/session";
import { StdioTransport } from "../src/protocol/stdioTransport";
import { SessionHost } from "../src/extension/sessionHost";

const binary = process.env.SUFFICE_BIN;
const maybe = binary ? describe : describe.skip;

maybe("real app-server", () => {
  let home: string;
  let rpc: JsonRpcConnection;
  let session: AppServerSession;

  beforeAll(async () => {
    home = mkdtempSync(join(tmpdir(), "suffice-ext-test-"));
    const transport = new StdioTransport({
      binary: binary!,
      cwd: home,
      env: { ...process.env, SUFFICE_HOME: home },
    });
    rpc = new JsonRpcConnection(transport);
    session = new AppServerSession(rpc);
    await session.initialize({ name: "suffice-vscode-test", title: null, version: "0.0.0" });
  }, 60_000);

  afterAll(() => {
    rpc?.close();
    try {
      rmSync(home, { recursive: true, force: true });
    } catch {
      // the server may still hold a file for a moment on Windows
    }
  });

  test("model/list returns the catalog", async () => {
    const models = await session.modelList({ includeHidden: true });
    expect(models.data.length).toBeGreaterThan(0);
    expect(models.data.some((m) => m.id === "glm-5.3-flash")).toBe(true);
  });

  test("config/read exposes the compaction keys", async () => {
    const config = await session.configRead({ includeLayers: false, cwd: home });
    expect(config.config).toHaveProperty("model_auto_compact_token_limit");
  });

  test("skills/list and thread/list answer", async () => {
    const skills = await session.skillsList({ cwds: [home] });
    expect(Array.isArray(skills.data)).toBe(true);
    const threads = await session.threadList({ limit: 5 });
    expect(Array.isArray(threads.data)).toBe(true);
  });

  test("collaborationMode/list is available with experimentalApi", async () => {
    const modes = await session.collaborationModeList();
    expect(modes.data.map((m) => m.mode)).toEqual(expect.arrayContaining(["plan", "default"]));
  });

  test("SessionHost starts, relays a request, reports stop", async () => {
    const statuses: string[] = [];
    const host = new SessionHost({
      status: (s) => statuses.push(s.state),
      notification: () => {},
      serverRequest: () => {},
      log: () => {},
    });
    await host.start({ binary: binary!, cwd: home, env: { ...process.env, SUFFICE_HOME: home }, version: "0.0.0" });
    expect(host.currentStatus.state).toBe("ready");
    const models = (await host.request("model/list", {})) as { data: unknown[] };
    expect(models.data.length).toBeGreaterThan(0);
    host.stop();
    await expect(host.request("model/list", {})).rejects.toThrow("not running");
    expect(statuses.slice(0, 2)).toEqual(["starting", "ready"]);
  }, 60_000);

  test("SessionHost without a binary reports noBinary", async () => {
    const host = new SessionHost({ status: () => {}, notification: () => {}, serverRequest: () => {}, log: () => {} });
    await host.start({ binary: undefined, cwd: home, env: process.env, version: "0" });
    expect(host.currentStatus).toEqual({ state: "noBinary" });
  });
});
