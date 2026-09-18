## License

This repository is licensed under the [Apache-2.0 License](../LICENSE).

Selling this, or anything built from it, is permitted by every licence below. What that obliges,
and the one licence here that obliges more than the others, is in
[commercial licensing](commercial-licensing.md).

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
| [OpenCode](https://github.com/sst/opencode) | MIT ([copy](../third_party/opencode/LICENSE)) | `codex-rs/core/src/tools/` | tool descriptions, output layouts and result limits, much of it verbatim |

### On the RTK entry

Worth stating plainly, because it is the only entry of its kind here: no RTK source was copied.
What was taken is a rule — *a compressed rendering must never cost more tokens than the raw output
it replaces* — together with the name RTK gives it. The comparison, the token estimator and the
tests in `never_worse` are this repository's own, written against this repository's own
`approx_token_count`.

It is recorded here anyway. Apache-2.0 asks for attribution when a work is derived from another,
and an idea taken deliberately from a named project, under the name that project gave it, is
closer to derivation than to coincidence. The cost of saying so is three lines.

### On the OpenCode entry

This one was wrong here until it was checked, and the correction is worth keeping visible.

The 52 in-source comments crediting OpenCode were treated as sufficient on the grounds that what
was taken was wording and numbers rather than code. But `tools/handlers/search_spec.rs` opens by
saying the search tool descriptions were *copied* from OpenCode and that everything not named in
the exception is *verbatim* — and OpenCode is MIT, which asks for its copyright and permission
notice in all copies, not for prose credit. So it is in `NOTICE` now, with a licence copy under
`third_party/opencode/`, like the rest.

The comments stay. They say which line came from where, which a notice file cannot.
