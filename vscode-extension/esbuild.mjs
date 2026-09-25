// Two bundles: the extension host (Node, CommonJS, `vscode` external) and the webview
// (browser IIFE with its CSS). Protocol types come from codex-rs via the tsconfig
// `@protocol/*` path and are type-only, so nothing from codex-rs lands in either bundle.
import * as esbuild from "esbuild";

const watch = process.argv.includes("--watch");
const production = process.argv.includes("--production");

const common = {
  bundle: true,
  sourcemap: !production,
  minify: production,
  logLevel: "info",
  tsconfig: "tsconfig.json",
};

const builds = [
  {
    ...common,
    entryPoints: ["src/extension/extension.ts"],
    outfile: "dist/extension.js",
    platform: "node",
    format: "cjs",
    target: "node20",
    external: ["vscode"],
  },
  {
    ...common,
    entryPoints: ["src/webview/main.tsx"],
    outfile: "dist/webview.js",
    platform: "browser",
    format: "iife",
    target: "es2022",
    jsx: "automatic",
    define: { "process.env.NODE_ENV": production ? '"production"' : '"development"' },
  },
];

if (watch) {
  for (const options of builds) {
    const ctx = await esbuild.context(options);
    await ctx.watch();
  }
} else {
  await Promise.all(builds.map((options) => esbuild.build(options)));
}
