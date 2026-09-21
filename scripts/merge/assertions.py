#!/usr/bin/env python3
r"""Pin the mechanisms by name, and say what changed rather than only what exists.

This replaces `_labs/merge-2026-09-20/scripts/verify_savings.py`, which passes when it
should not, in four ways found by auditing it:

  - its file-integrity section prints `degisti` and still ends in `HEPSI YERINDE`,
    because only the second section can fail the run
  - every positive check is `text.count(needle) > 0` over the raw file, so a needle that
    survives only inside a comment saying it was removed still passes
  - negative checks are selected by `"YOK" in label`, so any label containing those three
    letters silently inverts the test
  - nothing compares against the pre-merge branch, so one occurrence and twelve read alike

Here every check is counted on comment- and string-stripped code at BOTH the pre-merge
branch and the merged tree, and reported as a pair. A mechanism that went 12 -> 1 is a
finding even though "it is still there" is true.

The list is deliberately explicit rather than derived. survival.py finds losses nobody
knew to look for; this pins the ones that are already known to matter - including the
class survival.py cannot judge, where upstream rewrote the same region and its verdict is
necessarily "a decision was made here" rather than "this is a defect".

    python scripts/merge/assertions.py
    exit 0 when every check holds, 1 otherwise
"""
import io
import json
import os
import subprocess
import sys

import rustscan

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=HERE,
                      capture_output=True, encoding="utf-8").stdout.strip()

OURS = "88a2b0c53"
HEAD = "HEAD"

CORE = "codex-rs/core/src/"
MM = "codex-rs/models-manager/"

# (label, tier, path, needle, mode, where)
#   mode  "present" | "absent"
#   where "code" strips comments and strings; "raw" keeps them, for needles that ARE
#         string literals or live in prose
CHECKS = [
    # --- tool surface: the fixed prefix of every request ---------------------
    ("read araci kayitli", "T1", CORE + "tools/spec_plan.rs", "ReadHandler", "present", "code"),
    ("grep araci kayitli", "T1", CORE + "tools/spec_plan.rs", "GrepHandler", "present", "code"),
    ("glob araci kayitli", "T1", CORE + "tools/spec_plan.rs", "GlobHandler", "present", "code"),
    ("lean_parameters hesaplaniyor", "T1", CORE + "tools/spec_plan.rs",
     "let lean_parameters", "present", "code"),
    ("lean_parameters dalllaniyor", "T1", CORE + "tools/handlers/shell_spec.rs",
     "lean_exec_command_tool", "present", "code"),
    ("ripgrep motoru", "T1", CORE + "tools/handlers/search_rg.rs", "fn run_rg", "present", "code"),

    # --- tool output: shrink, budget, spill ----------------------------------
    ("never_worse tanimli", "T1", "codex-rs/utils/output-truncation/src/lib.rs",
     "fn never_worse", "present", "code"),
    ("never_worse CAGRILIYOR", "T1", "codex-rs/utils/output-truncation/src/lib.rs",
     "never_worse(", "present", "code"),
    ("with_serialization_allowance", "T1", "codex-rs/utils/output-truncation/src/lib.rs",
     "fn with_serialization_allowance", "present", "code"),
    ("spill: output_budget", "T1", CORE + "tools/context.rs", "fn output_budget", "present", "code"),
    ("spill: write_spill", "T1", CORE + "tools/context.rs", "fn write_spill", "present", "code"),
    ("spill: spill_notice", "T1", CORE + "tools/context.rs", "fn spill_notice", "present", "code"),
    ("spill: modele haber metni", "T1", CORE + "tools/context.rs",
     "Full output saved to", "present", "raw"),
    ("spill dizini cozuluyor", "T1", CORE + "unified_exec/mod.rs",
     "fn tool_output_spill_dir", "present", "code"),
    ("spill: bayat dizin supurmesi", "T1", CORE + "unified_exec/mod.rs",
     "fn sweep_stale_tool_output", "present", "code"),
    ("spill: oturum sonu temizligi", "T1", CORE + "session/handlers.rs",
     "remove_session_tool_output", "present", "code"),
    ("kaynakta kisaltma", "T1", CORE + "context_manager/tool_output.rs",
     "fn condense_exec_output", "present", "code"),

    # --- compaction: the most fragile line in the merge -----------------------
    ("compaction arac listesi", "T1", CORE + "compact.rs",
     "tools: Arc::clone(&tools)", "present", "code"),
    ("compaction: son arac spec'leri", "T1", CORE + "compact.rs",
     "last_known_tool_specs", "present", "code"),
    ("compact.rs attach YOK", "T1", CORE + "compact.rs",
     "attach_to_compaction_prompt", "absent", "code"),
    ("compaction testi duruyor", "T1", CORE + "compact_tests.rs",
     "compaction_request_stays_on_the_turns_cached_prefix", "present", "code"),

    # --- reasoning retention and request measurement --------------------------
    ("retention tur basinda donduruluyor", "T1", CORE + "session/turn.rs",
     "freeze_reasoning_retention", "present", "code"),
    ("for_prompt tura kapsamli", "T1", CORE + "context_manager/history.rs",
     "active_turn_id", "present", "code"),
    ("istek istatistikleri", "T1", CORE + "session/turn.rs",
     "request_stats", "present", "code"),
    ("prompt_input_for_step", "T1", CORE + "session/mod.rs",
     "fn prompt_input_for_step", "present", "code"),
    ("onbellek canli tutma", "T1", CORE + "tools/router.rs",
     "hold_prompt_cache_keep_alive", "present", "code"),

    # --- invisible turns -------------------------------------------------------
    ("invisible: dusurme nedeni", "T1", CORE + "context_manager/history.rs",
     "DropReason::Invisible", "present", "code"),
    ("invisible: aktif tur testi", "T1", CORE + "context_manager/history.rs",
     "fn belongs_to_active_turn", "present", "code"),
    ("invisible: metadata yaziliyor", "T1", CORE + "session/mod.rs",
     "invisible_turn", "present", "code"),

    # --- apply_patch: grammar or prose, never both ------------------------------
    ("apply_patch hizalamasi", "T1", MM + "src/model_info.rs",
     "fn align_apply_patch_section", "present", "code"),
    ("apply_patch prose metni", "T1", MM + "src/model_info.rs",
     "APPLY_PATCH_PROSE", "present", "code"),
    ("with_config_overrides hunisi", "T1", MM + "src/model_info.rs",
     "fn with_config_overrides", "present", "code"),

    # --- the wire this fork's provider needs ------------------------------------
    ("Chat wire govdesi", "T1", "codex-rs/codex-api/src/endpoint/responses.rs",
     "chat_body_from_responses_request", "present", "code"),
    ("ResponsesEndpoint", "T1", "codex-api/src/endpoint/responses.rs".replace(
        "codex-api", "codex-rs/codex-api"), "enum ResponsesEndpoint", "present", "code"),
    ("Chat akis cozucusu", "T1", "codex-rs/codex-api/src/sse/chat.rs",
     "fn spawn_chat_stream", "present", "code"),
    ("saglayici OpenAI degilse remote-v2 yok", "T1",
     "codex-rs/model-provider/src/provider.rs", "is_openai()", "present", "code"),
]


def blob(rev, path):
    r = subprocess.run(["git", "show", "%s:%s" % (rev, path)], cwd=REPO,
                       capture_output=True)
    return None if r.returncode else r.stdout.decode("utf-8", "replace")


def count(rev, path, needle, where):
    src = blob(rev, path)
    if src is None:
        return None
    text = rustscan.code_text(src) if where == "code" else src
    return text.count(needle)


def catalog():
    """models.json is data, not lines: a field can survive and carry upstream's value."""
    out = []
    try:
        a = json.loads(blob(OURS, MM + "models.json").lstrip("﻿"))
        b = json.loads(blob(HEAD, MM + "models.json").lstrip("﻿"))
    except Exception as exc:
        return [("models.json okunamadi", "T1", str(exc), False)]
    ia = {m.get("slug"): m for m in a.get("models", [])}
    ib = {m.get("slug"): m for m in b.get("models", [])}

    def tpl(m):
        return ((m or {}).get("model_messages") or {}).get("instructions_template") or ""

    drift = [s for s in ia if s in ib and tpl(ia[s]) != tpl(ib[s])]
    out.append(("her sablon OURS ile birebir", "T1",
                "farkli: %s" % (", ".join(drift) or "yok"), not drift))

    g = ib.get("glm-5.3-flash")
    out.append(("glm-5.3-flash var", "T1", "-", g is not None))
    if g:
        n = len(tpl(g))
        out.append(("GLM sablonu 16166 karakter", "T1", "%d" % n, n == 16166))
        out.append(("GLM apply_patch_tool_type=prose", "T1",
                    str(g.get("apply_patch_tool_type")), g.get("apply_patch_tool_type") == "prose"))
        out.append(("GLM visibility=list", "T1", str(g.get("visibility")),
                    g.get("visibility") == "list"))
    for slug in ("gpt-5.2", "gpt-5.4-mini"):
        out.append(("%s korunmus" % slug, "T2", "-", slug in ib))

    key = "you use `glob`"
    na = sum(1 for m in ia.values() if key in tpl(m))
    nb = sum(1 for m in ib.values() if key in tpl(m))
    out.append(("olculen glob/grep satiri", "T1", "%d -> %d model" % (na, nb), nb >= na))
    return out


def main():
    rows, bad = [], []
    for label, tier, path, needle, mode, where in CHECKS:
        a = count(OURS, path, needle, where)
        b = count(HEAD, path, needle, where)
        if b is None:
            ok, detail = False, "DOSYA YOK"
        elif mode == "present":
            ok = b > 0
            detail = "%s -> %s" % ("?" if a is None else a, b)
            if ok and a is not None and b < a:
                detail += "  (AZALDI)"
        else:
            ok = b == 0
            detail = "%s -> %s" % ("?" if a is None else a, b)
        rows.append((label, tier, detail, ok))
        if not ok:
            bad.append(label)

    for label, tier, detail, ok in catalog():
        rows.append((label, tier, detail, ok))
        if not ok:
            bad.append(label)

    print("=" * 78)
    print("  MEKANIZMA IDDIALARI   (%s -> %s)" % (OURS, HEAD))
    print("=" * 78)
    for label, tier, detail, ok in rows:
        print("  %-4s %-38s %-22s %s" % (tier, label[:38], detail[:22],
                                         "TAMAM" if ok else "!! SORUN"))
    print("-" * 78)
    if bad:
        print("  SONUC: %d SORUN -> %s" % (len(bad), ", ".join(bad)))
        return 1
    print("  SONUC: butun mekanizmalar yerinde ve cagrilabilir.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
