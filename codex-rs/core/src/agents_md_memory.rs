//! Refreshes the project's durable memory in `AGENTS.md` right before compaction.
//!
//! Compaction is the moment a conversation's detail is about to be thrown away, which makes it
//! the natural moment to persist whatever is worth keeping past the end of the session. Suffice
//! already loads `AGENTS.md` as user instructions at the start of every session, so a file the
//! agent keeps current is what carries today's conversation into tomorrow's.
//!
//! This is not a special mechanism. It is the ordinary agent loop, given the ordinary toolset,
//! asked to update a file — the same thing that happens when a user types "update AGENTS.md".
//! The agent reads, patches, and reports failures through the paths it already uses for every
//! other edit, so there is nothing here to define a bespoke tool, a retry budget, or a private
//! notion of failure for.
//!
//! The one thing that is special is that the turn is *invisible*: its items are recorded, so the
//! agent sees its own reads and edits while the turn runs, but they carry this turn's id as their
//! invisibility owner and are therefore dropped from every later prompt. The compaction that
//! follows passes no active turn id, so it sees history byte-identical to what it would have seen
//! had this never run.

use std::sync::Arc;

use crate::agents_md::DEFAULT_AGENTS_MD_FILENAME;
use crate::agents_md::agents_md_paths;
use crate::config::Config;
use crate::responses_metadata::CodexResponsesRequestKind;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn::run_sampling_request;
use crate::session::turn_context::TurnContext;
use crate::turn_diff_tracker::TurnDiffTracker;
use crate::util::backoff;
use codex_exec_server::ExecutorFileSystem;
use codex_features::Feature;
use codex_file_system::FileSystemSandboxContext;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_utils_path_uri::PathUri;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::warn;

/// Asks the agent to write what this conversation established into `AGENTS.md`.
///
/// Never fails the caller: compaction must proceed whether or not this works.
#[tracing::instrument(name = "agents_md.memory_refresh", skip_all)]
pub(crate) async fn refresh_agents_md_before_compaction(
    sess: &Arc<Session>,
    turn_context: &TurnContext,
) {
    if !turn_context.config.features.enabled(Feature::ProjectMemory) {
        return;
    }
    if turn_context.config.active_project.is_untrusted() {
        return;
    }
    let Some(target) = memory_file_path(turn_context).await else {
        return;
    };

    run_memory_turn(sess, turn_context, codex_prompts::agents_md_memory_prompt(&target)).await;

    // The manager's cache key is the environment selections and the trust level, neither of which
    // changed here, so without this the file the agent just wrote stays invisible for the rest of
    // the session.
    sess.services.agents_md_manager.invalidate().await;
}

/// The memory file to update, spelled the way the agent should spell it: relative to the turn's
/// working directory.
///
/// Discovery is Suffice's own, so `AGENTS.override.md` and `project_doc_fallback_filenames` are
/// honoured. The deepest hit is the one closest to cwd, and the one a future session loads last.
/// With no file on disk yet there is nothing to discover, so name the default and let the agent
/// create it.
async fn memory_file_path(turn_context: &TurnContext) -> Option<String> {
    let env = turn_context.environments.primary()?;
    let fs = env.environment.get_filesystem();
    let sandbox = env.sandbox_context(/*additional_permissions*/ None);
    Some(
        resolve_memory_file_path(&turn_context.config, env.cwd(), fs.as_ref(), Some(&sandbox))
            .await,
    )
}

async fn resolve_memory_file_path(
    config: &Config,
    cwd: &PathUri,
    fs: &dyn ExecutorFileSystem,
    sandbox: Option<&FileSystemSandboxContext>,
) -> String {
    let discovered = match agents_md_paths(config, cwd, fs, sandbox).await {
        Ok(paths) => paths.last().cloned(),
        Err(err) => {
            warn!("project memory discovery failed: {err}");
            None
        }
    };
    discovered
        .and_then(|target| target.relative_path_from(cwd))
        .unwrap_or_else(|| DEFAULT_AGENTS_MD_FILENAME.to_string())
}

/// Runs one ordinary agent turn, invisibly, seeded with `prompt`.
///
/// This is the body of `run_turn`'s sampling loop and nothing else. `run_turn` itself cannot be
/// reused because the first thing it does is `run_pre_sampling_compact`, which would re-enter the
/// compaction that is already running. Every other thing it does around the loop is skipped on
/// purpose:
///
/// | In `run_turn`, not here | Why |
/// |---|---|
/// | `run_pre_sampling_compact` | Would re-enter the compaction that is already running |
/// | `run_hooks_and_record_inputs` / stop hooks / `after_agent` | Running a user-prompt hook for a message the user never sent is wrong, and a Stop hook can block the turn and make it continue |
/// | `build_skills_and_plugins` | Injecting skills to edit one file is tokens for nothing |
/// | pending input drain / mailbox | User input belongs to the next real turn, not to this one |
/// | `record_context_updates_and_set_reference_context_item`, `record_step_world_state_if_changed` | The parent turn already recorded these; recording again would re-inject `AGENTS.md` mid-turn |
/// | time / rollout reminder records | Would pollute history |
/// | context-window rollover | We are inside compaction and cannot compact again |
///
/// The last row matters: this turn runs above the ~80k threshold by construction. If the request
/// overflows the context window, `run_sampling_request` returns an error, it is logged, this
/// function returns, and compaction proceeds normally. The model call is not retried here because
/// `run_sampling_request` already does it (`stream_max_retries`, default 5) and drops the errors
/// retrying cannot fix. Only `capture_step_context` is retried, because it is local and costs
/// nothing. Still deliberately no token cap and no iteration limit — the agent gets the same loop
/// it gets for any other file edit.
async fn run_memory_turn(sess: &Arc<Session>, parent: &TurnContext, prompt: String) {
    let turn_context = Arc::new(parent.invisible_child(uuid::Uuid::new_v4().to_string()));
    sess.record_conversation_items(
        &turn_context,
        &[ResponseItem::Message {
            id: None,
            // Same shape compaction uses for its own request (`compact.rs`): role "user", with the
            // prompt itself written as a system procedure. The Chat adapter forwards roles verbatim
            // (`codex-api/src/requests/chat.rs`), so "developer" would reach the provider raw, and
            // it would also break `anchor_trailing_reasoning`, which locates the last "user"
            // message.
            role: "user".to_string(),
            content: vec![ContentItem::InputText { text: prompt }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        }],
    )
    .await;

    // Child of the compact task's own token, so Ctrl-C during compaction stops this turn too.
    let cancellation_token = sess
        .active_turn
        .lock()
        .await
        .as_ref()
        .and_then(|active| active.task.as_ref())
        .map(|task| task.cancellation_token.child_token())
        .unwrap_or_default();
    let turn_diff_tracker = Arc::new(Mutex::new(TurnDiffTracker::new()));
    let mut client_session = sess.services.model_client.new_session();
    let responses_metadata = sess
        .responses_metadata(turn_context.as_ref(), CodexResponsesRequestKind::Memory)
        .await;

    loop {
        if cancellation_token.is_cancelled() {
            return;
        }
        let Some(step_context) =
            capture_step_context_with_retry(sess, &turn_context, &cancellation_token).await
        else {
            return;
        };
        // `Some(sub_id)` is what lets this turn see its own items; every later prompt passes a
        // different id, or none, and so sees none of them.
        let input = sess.clone_history().await.for_prompt(
            &step_context.settings.model_info.input_modalities,
            Some(turn_context.sub_id.as_str()),
        );
        let result = run_sampling_request(
            Arc::clone(sess),
            Arc::clone(&step_context),
            Arc::clone(&turn_context.extension_data),
            Arc::clone(&turn_diff_tracker),
            &mut client_session,
            &responses_metadata,
            input,
            cancellation_token.child_token(),
        )
        .await;
        match result {
            Ok((sampling_request, _)) if sampling_request.needs_follow_up => continue,
            Ok(_) => return,
            Err(err) => {
                warn!("project memory turn failed: {err}");
                return;
            }
        }
    }
}

/// Attempts for the local, cost-free part of the turn. The model call is not retried here:
/// `run_sampling_request` already retries it `stream_max_retries` times (default 5) and filters out
/// the errors that retrying cannot fix.
const STEP_CONTEXT_ATTEMPTS: u64 = 5;

/// Each step gets its own budget rather than sharing one across the turn, which would leave a long
/// tool loop with nothing left by the time it mattered.
async fn capture_step_context_with_retry(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    cancellation_token: &CancellationToken,
) -> Option<Arc<StepContext>> {
    let mut attempt = 1;
    loop {
        match sess
            .capture_step_context(Arc::clone(turn_context), cancellation_token)
            .await
        {
            Ok(step_context) => return Some(step_context),
            Err(err) => {
                if attempt >= STEP_CONTEXT_ATTEMPTS {
                    warn!("project memory turn could not start after {attempt} attempts: {err}");
                    return None;
                }
                warn!("project memory step context attempt {attempt} failed, retrying: {err}");
                // Interruptible: an uncancellable sleep here would hold Ctrl-C for the whole backoff.
                tokio::select! {
                    () = tokio::time::sleep(backoff(attempt)) => {}
                    () = cancellation_token.cancelled() => return None,
                }
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigBuilder;
    use codex_exec_server::LOCAL_FS;
    use core_test_support::PathBufExt;
    use core_test_support::TempDirExt;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    async fn config_for(root: &TempDir, fallbacks: &[&str]) -> Config {
        let codex_home = TempDir::new().expect("codex home tempdir");
        let mut config = ConfigBuilder::default()
            .codex_home(codex_home.path().to_path_buf())
            .build()
            .await
            .expect("test config");
        config.cwd = root.abs();
        config.project_doc_fallback_filenames = fallbacks.iter().map(ToString::to_string).collect();
        config
    }

    #[tokio::test]
    async fn agents_md_memory_path_is_the_deepest_discovery_relative_to_cwd() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(tmp.path().join(".git")).expect("project root marker");
        std::fs::write(tmp.path().join("AGENTS.md"), "root doc").expect("root doc");
        let nested = tmp.path().join("nested");
        std::fs::create_dir(&nested).expect("nested dir");
        std::fs::write(nested.join("CONTEXT.md"), "nested doc").expect("nested doc");

        let config = config_for(&tmp, &["CONTEXT.md"]).await;
        let cwd = PathUri::from_abs_path(&nested.abs());

        // The root's AGENTS.md is discovered first but is not under cwd; the deepest hit wins and
        // is spelled relative to cwd.
        assert_eq!(
            resolve_memory_file_path(&config, &cwd, LOCAL_FS.as_ref(), /*sandbox*/ None).await,
            "CONTEXT.md"
        );
    }

    #[tokio::test]
    async fn agents_md_memory_path_falls_back_to_the_default_filename() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = config_for(&tmp, /*fallbacks*/ &[]).await;
        let cwd = PathUri::from_abs_path(&config.cwd);

        assert_eq!(
            resolve_memory_file_path(&config, &cwd, LOCAL_FS.as_ref(), /*sandbox*/ None).await,
            DEFAULT_AGENTS_MD_FILENAME
        );
    }
}
