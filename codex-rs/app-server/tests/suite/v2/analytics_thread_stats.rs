use std::fs;
use std::time::Duration;

use anyhow::Result;
use app_test_support::TestAppServer;
use codex_app_server_protocol::AnalyticsThreadStatsParams;
use codex_app_server_protocol::AnalyticsThreadStatsResponse;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// A sidecar as `codex-core`'s request-stats module writes it: one header
/// line, then one line per request. The corrupt line stands in for a row
/// interrupted mid-write and must be skipped, not fail the read.
const SIDECAR: &str = concat!(
    r#"{"thread_id":"t-live","started_at":"2026-09-20T10:00:00Z","model":"glm-5.3-flash","provider":"Z.ai","reasoning_effort":"high","codex_version":"0.1.0","features":[],"requests_per_window":160.0}"#,
    "\n",
    r#"{"sequence":1,"timestamp":"2026-09-20T10:00:05Z","turn_id":"turn-1","response_id":null,"invisible":false,"input_tokens":1000,"cached_input_tokens":0,"output_tokens":50,"reasoning_output_tokens":10,"total_tokens":1050,"dropped_call_ids":[],"tool_calls":1,"duration_ms":900}"#,
    "\n",
    r#"{"sequence":2,"timestamp":"2026-09-20T10:01:00Z","turn_id":"turn-1","response_id":null,"invisible":true,"input_tokens":1000,"cached_input_tokens":990,"output_tokens":1,"reasoning_output_tokens":0,"total_tokens":1001,"dropped_call_ids":[],"tool_calls":0,"duration_ms":80}"#,
    "\n",
    r#"{"sequence":3,"timestamp":"2026-09-20T10:02:00Z","turn_id":"turn-2","response_id":null,"invisible":false,"input_tokens":2000,"cached_input_tokens":1800,"output_tokens":120,"reasoning_output_tokens":30,"total_tokens":2120,"dropped_call_ids":[],"tool_calls":2,"duration_ms":1500}"#,
    "\n",
    r#"{"sequence":4,"timestamp":"2026-09-2"#,
    "\n",
);

#[tokio::test]
async fn analytics_thread_stats_aggregates_the_sidecar_and_omits_absences() -> Result<()> {
    let codex_home = TempDir::new()?;
    let analytics_dir = codex_home.path().join("analytics");
    fs::create_dir_all(&analytics_dir)?;
    fs::write(analytics_dir.join("t-live.jsonl"), SIDECAR)?;
    // Present on disk, but reachable only through a path-shaped id that the
    // endpoint must refuse to resolve.
    fs::write(analytics_dir.join("t-other.jsonl"), SIDECAR)?;

    let mut app_server = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;

    let params = AnalyticsThreadStatsParams {
        thread_ids: vec![
            "t-live".to_string(),
            "t-missing".to_string(),
            "../analytics/t-other".to_string(),
        ],
    };
    let request_id = app_server
        .send_request("analytics/threadStats", Some(serde_json::to_value(params)?))
        .await?;
    let AnalyticsThreadStatsResponse { data } =
        timeout(DEFAULT_TIMEOUT, app_server.read_response(request_id)).await??;

    assert_eq!(data.len(), 1, "missing and path-shaped ids must be omitted");
    let stats = &data[0];
    assert_eq!(stats.thread_id, "t-live");
    assert_eq!(stats.model.as_deref(), Some("glm-5.3-flash"));
    assert_eq!(stats.provider.as_deref(), Some("Z.ai"));
    assert_eq!(stats.started_at.as_deref(), Some("2026-09-20T10:00:00Z"));
    assert_eq!(stats.last_timestamp.as_deref(), Some("2026-09-20T10:02:00Z"));
    assert_eq!(stats.requests, 3, "the corrupt trailing row is skipped");
    assert_eq!(stats.invisible_requests, 1);
    assert_eq!(stats.input_tokens, 4000);
    assert_eq!(stats.cached_input_tokens, 2790);
    assert_eq!(stats.output_tokens, 171);
    assert_eq!(stats.reasoning_output_tokens, 40);
    assert!(!stats.timeline_truncated);
    assert_eq!(stats.timeline.len(), 3);
    assert_eq!(stats.timeline[2].sequence, 3);
    assert_eq!(stats.timeline[2].cached_input_tokens, 1800);
    Ok(())
}
