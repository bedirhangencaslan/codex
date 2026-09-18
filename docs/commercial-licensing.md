# Commercial licensing

Written for whoever prepares this product for sale, and for the lawyer they hand it to. It is an
inventory of what this repository actually contains and what each part obliges, not legal advice.
Two items at the end need a lawyer and are marked.

## The short version

**You may sell this.** Every inherited licence here — Apache-2.0 and MIT — permits commercial use,
distribution and sale, including of modified versions, and none requires you to publish your own
changes. One exception is the Linux-only `bwrap` executable; see below.

**You may not put the whole repository under a licence of your own.** The inherited code stays
under the licence it arrived with. What you may license on your own terms is *your own* work: the
code this fork added. Apache-2.0 §4 says so explicitly — you may add your own copyright statement
to your modifications and offer them under any terms — but the grant on the code you did not write
travels with it to your customers.

So the product ships as: your additions under your terms, on top of an Apache-2.0 base whose
licence and NOTICE you carry along.

## What the base is

`LICENSE` is Apache-2.0 and `NOTICE` names OpenAI Codex, because this is a fork of
[openai/codex](https://github.com/openai/codex). Every crate inherits `license = "Apache-2.0"` from
`[workspace.package]`, and CI enforces the inheritance
(`.github/scripts/verify_cargo_workspace_manifests.py`).

## What shipping obliges, concretely

### Apache-2.0 (the base, and RTK)

1. Include a copy of the licence — `LICENSE`, already there.
2. Keep the existing copyright, patent, trademark and attribution notices.
3. Carry `NOTICE` and reproduce its contents in your distribution, in the documentation or in a
   display the product shows.
4. **State that you changed the files you changed.** This fork has changed a great deal and says
   so only in its commit history. A one-line statement in `NOTICE` or a `CHANGES` file satisfies it.

### MIT (OpenCode, WezTerm, path-absolutize, Ratatui)

One obligation, and it is absolute: the copyright notice and the permission notice must be included
in all copies or substantial portions. Copies of each licence are under `third_party/`, and each
project is named in `NOTICE`.

Worth knowing how this nearly went wrong, because it is the kind of thing that is invisible until
someone audits you. `core/src/tools/handlers/search_spec.rs` opens with *"Specs for the two search
tools, copied from OpenCode… Everything else is verbatim"* — and until this file was written,
OpenCode appeared nowhere in `NOTICE` and had no licence copy, only 52 comments crediting it in
prose. Prose credit is not what MIT asks for.

### LGPL-2.0-or-later (bubblewrap) — **Linux builds only**

`codex-rs/vendor/bubblewrap/` holds the upstream tree, and `codex-rs/bwrap/build.rs` compiles it
with `cc` into a **separate executable named `bwrap`**. The build script returns early unless
`CARGO_CFG_TARGET_OS == "linux"`, so a Windows or macOS build contains none of it.

Two things follow:

- A **Windows-only or macOS-only product** ships no LGPL code. The obligation does not arise in the
  binary you sell.
- A **Linux product** ships `bwrap`, which is a derivative work of bubblewrap. LGPL-2.0 requires,
  among other things, that recipients be able to relink the work against a modified bubblewrap, and
  that they be offered the source. Because the source is vendored in this repository, shipping the
  source alongside is the straightforward way to satisfy it; shipping only a binary is not.

`CODEX_SKIP_BWRAP_BUILD` exists and skips the compile. Whether the product still functions without
`bwrap` on Linux is a question about the sandbox, not about licensing, and has not been tested here.

## What the caps and layouts are, licence-wise

Several numbers and sentences in `core/src/tools/` are OpenCode's — the 100-result limit, the 51,200
byte output cap, the `grep` layout, the `exec_command` usage notes — and are credited in the comment
next to each. Those are now covered by the MIT notice like the rest of OpenCode's material.

The never-worse guard in `utils/output-truncation/` is a different case: the invariant and its name
are RTK's, the implementation is this repository's own. It is recorded in `NOTICE` anyway, because
an idea taken deliberately from a named project, under that project's name for it, is closer to
derivation than to coincidence, and three lines is a cheap way not to have that argument.

## Two things that need a lawyer

**1. The licence your additions ship under, and the agreement your customers sign.** A proprietary
licence, an EULA, warranty disclaimers, limitation of liability, export terms and the boundary
between "your additions" and "the Apache base" are drafting work, and getting the boundary wrong is
the expensive kind of mistake. Nothing in this document is a substitute for that.

**2. Whether you ship Linux at all, and how.** The LGPL question above is answerable, but it is a
real question with real conditions, and the answer changes what you must distribute. Decide it
before the first sale rather than after.

## Also worth raising with them

- **Trademarks.** Apache-2.0 §6 grants no trademark rights. "Codex" is OpenAI's product name and
  this repository's `NOTICE` still reads "OpenAI Codex". A product sold under a name that suggests
  an OpenAI product is a trademark question, not a copyright one, and the licence is silent on it.
- **Model provider terms.** What the product does is send prompts to a model API. Those providers'
  terms of service govern commercial resale of access, and they are contracts, not licences — they
  are not in this repository and not covered by anything here.
- **`codex-rs/deny.toml`** lists the licences allowed in the dependency graph and CI blocks
  additions outside it. It is the right place to look for what the *crates* oblige, and it says
  nothing about hand-copied source such as the four files from WezTerm.
