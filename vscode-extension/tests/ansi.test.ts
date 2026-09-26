// Command output is shown in the webview, not a terminal, so escape sequences must be interpreted
// here: colours kept, every other control sequence gone.
import { parseAnsi, stripAnsi } from "../src/shared/ansi";

const ESC = "\x1b";

describe("terminal output", () => {
  test("uvicorn's startup lines lose every escape sequence and keep their colours", () => {
    const raw =
      `${ESC}[32mINFO${ESC}[0m:     Waiting for application startup.${ESC}[31X${ESC}[32m\n` +
      `INFO${ESC}[m:     Application startup complete.${ESC}[34X${ESC}[32m\n` +
      `INFO${ESC}[m:     Uvicorn running on ${ESC}[1m${ESC}[97mhttp://127.0.0.1:8000${ESC}[m (Press CTRL+C to quit)`;
    expect(stripAnsi(raw)).toBe(
      "INFO:     Waiting for application startup.\nINFO:     Application startup complete.\nINFO:     Uvicorn running on http://127.0.0.1:8000 (Press CTRL+C to quit)",
    );
    const segments = parseAnsi(raw);
    expect(segments[0]).toEqual({ text: "INFO", style: { fg: "var(--sf-success)" } });
    expect(segments.find((s) => s.text === "http://127.0.0.1:8000")!.style).toEqual({ bold: true, fg: "var(--sf-text)" });
    expect(stripAnsi(raw)).not.toMatch(/[\x00-\x08\x0b-\x1f]/);
  });

  test("256 and true colours, background, reset of single attributes", () => {
    const [a, b, c] = parseAnsi(`${ESC}[38;5;196;1mA${ESC}[22;48;2;0;0;255mB${ESC}[39;49mC`);
    expect(a).toEqual({ text: "A", style: { fg: "rgb(255,0,0)", bold: true } });
    expect(b!.style).toEqual({ fg: "rgb(255,0,0)", bold: false, dim: false, bg: "rgb(0,0,255)" });
    expect(c!.style).toEqual({ bold: false, dim: false });
  });

  test("cursor moves, OSC titles and hyperlinks, charset switches are dropped", () => {
    const raw = `${ESC}]0;title${ESC}\\${ESC}[2K${ESC}[1G${ESC}(Bok ${ESC}]8;;https://x.y${ESC}\\link${ESC}]8;;${ESC}\\ ${ESC}=done`;
    expect(stripAnsi(raw)).toBe("ok link done");
  });

  test("a progress bar redrawn with carriage returns shows its last state", () => {
    expect(stripAnsi("Downloading  10%\rDownloading  50%\rDownloading 100%\r\nnext line")).toBe("Downloading 100%\nnext line");
  });

  test("plain text, tabs and a lone ESC at the end", () => {
    expect(parseAnsi("a\tb")).toEqual([{ text: "a\tb", style: {} }]);
    expect(stripAnsi(`half${ESC}`)).toBe("half");
    expect(stripAnsi(`half${ESC}[`)).toBe("half");
  });
});
