## License

This repository is licensed under the [Apache-2.0 License](../LICENSE).

## Third-party code

Borrowed code carries its upstream licence at the top of the file that holds it, and, where the
upstream ships one, a copy of that licence under `third_party/<project>/`. The list below is what
a reader is most likely to want to find; [`NOTICE`](../NOTICE) is the file that records it
formally.

| Project | Licence | Where it lives | What was taken |
|---|---|---|---|
| [Ratatui](https://github.com/ratatui/ratatui) | MIT | `codex-rs/tui/src/custom_terminal.rs` | `ratatui::Terminal`, derived |
| [WezTerm](https://github.com/wezterm/wezterm) | MIT ([copy](../third_party/wezterm/LICENSE)) | `codex-rs/utils/pty/src/win/` | four files, copied |
| [path-absolutize](https://github.com/magiclen/path-absolutize) | MIT | `codex-rs/utils/absolute-path/src/absolutize.rs` | one function, adapted |
| [bubblewrap](https://github.com/containers/bubblewrap) | LGPL-2.0+ ([copy](../codex-rs/vendor/bubblewrap/COPYING)) | `codex-rs/vendor/bubblewrap/` | the upstream tree, vendored and built |
| [RTK](https://github.com/rtk-ai/rtk) | Apache-2.0 ([copy](../third_party/rtk/LICENSE)) | `codex-rs/utils/output-truncation/src/lib.rs` | the never-worse guard — **the invariant is theirs, the implementation is ours** |

### On the RTK entry

Worth stating plainly, because it is the only entry of its kind here: no RTK source was copied.
What was taken is a rule — *a compressed rendering must never cost more tokens than the raw output
it replaces* — together with the name RTK gives it. The comparison, the token estimator and the
tests in `never_worse` are this repository's own, written against this repository's own
`approx_token_count`.

It is recorded here anyway. Apache-2.0 asks for attribution when a work is derived from another,
and an idea taken deliberately from a named project, under the name that project gave it, is
closer to derivation than to coincidence. The cost of saying so is three lines.

### Ideas credited in source only

Several tool descriptions, output layouts and limits in `codex-rs/core/src/tools/` are taken from
[OpenCode](https://github.com/sst/opencode) and credited in the comment next to the line they
shaped — for example the 51,200-byte output cap in `tools/context.rs`, the `grep` result layout in
`tools/handlers/search.rs`, and the `exec_command` usage notes in `tools/handlers/shell_spec.rs`.
Those are wording and numbers rather than code, so they carry no licence header and no `NOTICE`
entry; the comments are the record.
