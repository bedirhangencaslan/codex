// Spawns `suffice app-server` and exposes its stdout as lines. stdio is the app-server's
// default transport (codex-rs/cli/src/main.rs, `app-server --listen stdio://`); stderr carries
// its logs, which go to the caller's log sink rather than being parsed.
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import type { Transport } from "./jsonrpc";

export interface SpawnOptions {
  binary: string;
  /** Global `-c key=value` overrides, placed before the subcommand like the test client does. */
  configOverrides?: string[];
  cwd?: string;
  env?: NodeJS.ProcessEnv;
  onStderr?: (text: string) => void;
}

/** Splits a byte stream into lines, tolerating chunks that end mid-line and CRLF endings. */
export class LineSplitter {
  private buffer = "";

  push(chunk: string, emit: (line: string) => void): void {
    this.buffer += chunk;
    let newline = this.buffer.indexOf("\n");
    while (newline !== -1) {
      const line = this.buffer.slice(0, newline).replace(/\r$/, "");
      this.buffer = this.buffer.slice(newline + 1);
      if (line) emit(line);
      newline = this.buffer.indexOf("\n");
    }
  }

  flush(emit: (line: string) => void): void {
    const rest = this.buffer.replace(/\r$/, "");
    this.buffer = "";
    if (rest) emit(rest);
  }
}

export class StdioTransport implements Transport {
  private readonly child: ChildProcessWithoutNullStreams;
  private readonly lineListeners: Array<(line: string) => void> = [];
  private readonly closeListeners: Array<(reason: string) => void> = [];
  private closed = false;

  constructor(options: SpawnOptions) {
    const args: string[] = [];
    for (const override of options.configOverrides ?? []) args.push("-c", override);
    args.push("app-server");

    this.child = spawn(options.binary, args, {
      cwd: options.cwd,
      env: options.env ?? process.env,
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
    });

    const splitter = new LineSplitter();
    this.child.stdout.setEncoding("utf8");
    this.child.stdout.on("data", (chunk: string) => {
      splitter.push(chunk, (line) => this.lineListeners.forEach((l) => l(line)));
    });
    this.child.stdout.on("end", () => {
      splitter.flush((line) => this.lineListeners.forEach((l) => l(line)));
    });
    this.child.stderr.setEncoding("utf8");
    this.child.stderr.on("data", (chunk: string) => options.onStderr?.(chunk));

    this.child.on("error", (error) => this.finish(`failed to start ${options.binary}: ${error.message}`));
    this.child.on("exit", (code, signal) =>
      this.finish(signal ? `app-server killed by ${signal}` : `app-server exited with code ${code}`),
    );
  }

  get pid(): number | undefined {
    return this.child.pid;
  }

  send(line: string): void {
    if (this.closed || !this.child.stdin.writable) return;
    this.child.stdin.write(line + "\n");
  }

  onLine(listener: (line: string) => void): void {
    this.lineListeners.push(listener);
  }

  onClose(listener: (reason: string) => void): void {
    this.closeListeners.push(listener);
  }

  close(): void {
    if (this.closed) return;
    this.child.stdin.end();
    // Give the server a moment to flush and exit on EOF before forcing it.
    const timer = setTimeout(() => {
      if (this.child.exitCode === null) this.child.kill();
    }, 1500);
    timer.unref?.();
  }

  private finish(reason: string): void {
    if (this.closed) return;
    this.closed = true;
    this.closeListeners.forEach((l) => l(reason));
  }
}
