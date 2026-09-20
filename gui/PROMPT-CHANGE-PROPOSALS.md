# Prompt Change Proposals — planned, never applied

This file is the quarantine zone required by the project owner: any idea from
GUI work that would change **model-visible tokens** (system prompt, tool specs,
skill instruction blocks, injected input items) is written up here as if it
were going to ship — full plan, expected effect, measurement design — and is
**not** implemented. Every entry waits for Bedirhan's explicit approval, because
`_sim/FINDINGS.md` showed these tokens dominate both behaviour and cost.

Rules of this file:

1. Nothing here may be turned into code, config-default change, or template
   edit before written approval — not even "obviously harmless" wording.
2. Every proposal must carry a measurement plan that follows the three rig
   rules from `_sim/FINDINGS.md` §3: interleaved arms, side-by-side baseline,
   a positive control of the right kind.
3. What the GUI is allowed to use without approval: existing documented
   mechanisms only — `skills/list`, `thread/list`, `thread/start`
   (incl. `ephemeral: true`), `turn/start` with the documented `skill` input
   item shape, and the existing `include_skill_instructions` config path.

---

## Proposal 1 — Skill instruction block: per-skill inclusion instead of all-or-nothing

- **Status:** DRAFT — awaiting review. Not implemented.
- **What would change:** today `[skills] include_instructions` is a single
  boolean; the prompt either carries the whole skills block (5,592 chars at the
  time of FINDINGS §8) or none of it. The Skills screen would ideally include
  only the *enabled* skills' instruction lines, shrinking the block instead of
  deleting it.
- **Model-visible delta:** the skills block's content varies per project;
  prefix bytes change whenever the selection changes (a cache break per edit).
- **Expected effect:** keeps the FINDINGS §6 result (skills block was the only
  transplant surviving both directions) while letting users retain the one or
  two skills they actually use.
- **Measurement plan:** probe.py on the captured HEAD prefix; arms =
  {block-off (current), block-full, block-selected(2 skills)}; 3 reps,
  round-robin, `oc+counts`-style lifting control on the fork base; decision
  metric = bytes-returned-per-read and frac-whole-file, exactly as §7.
- **Risk:** selection edits invalidate the prompt cache mid-session; must be
  batched to session start (the GUI already only writes config, which loads at
  session start, so the break lands once).

## Proposal 2 — `defaultPrompt` text for the Commands screen "Try" flow

- **Status:** DRAFT — awaiting review. Not implemented.
- **What would change:** when the user hits "Try" on a command card, the GUI
  starts an ephemeral thread and sends the command. The *text* we send is
  model input. Plan: send exactly the command string (`/status`) with no
  added framing. Any richer template ("You are in a tutorial…") is a prompt
  change and needs approval + measurement first.
- **Model-visible delta today if approved as stated:** none beyond what a
  TUI user typing the same command already produces — which is why this is
  the proposed default.

---

*Add new proposals above this line, newest first inside each section.*
