# Suffice GUI — desktop shell (Tauri)

A deliberately dumb frame around the `gui/web` bundle: no IPC commands, no
filesystem access, no model-visible surface. Data still flows over the
app-server WebSocket (`?ws=…`), identical to the browser build, so everything
the prompt-isolation rule guarantees for the web app holds here byte for byte.

This crate is standalone (`[workspace]` table) so the codex-rs Cargo workspace
and the Bazel module never absorb the Tauri dependency tree; `gui` is also
listed in `.bazelignore`.

## Run

```bash
cargo install tauri-cli --version '^2'   # once
cd gui/desktop/src-tauri
cargo tauri dev      # dev server + window
cargo tauri build    # release binary (enable [bundle] first for installers)
```

`bundle.active` is `false` until we add real icons; `cargo check` / `cargo
tauri dev` need none.
