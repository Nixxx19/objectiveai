//! `read_latest_continuation` — walk backwards through an agent's
//! request history (newest-first) and return the first turn that has
//! a continuation `.txt` file on disk.
//!
//! Used by `objectiveai agents message`'s fallback path: if the live
//! per-agent socket is unreachable, we resume the agent's most recent
//! **completed** conversation via continuation instead of dropping
//! the message on the floor. "Most recent completed" — not "most
//! recent attempted" — matters because the newest request may still
//! be streaming, was cancelled, or never produced a continuation;
//! we'd rather resume an earlier finished turn than refuse to deliver.

use serde::{Deserialize, Serialize};

use crate::filesystem::{Client, Error};

/// Everything needed to POST a continuation that re-enters an agent's
/// most recent completed conversation.
///
/// Field-set mirrors the inline-on-disk shape of
/// [`crate::agent::completions::request::AgentCompletionCreateParamsLog`]:
/// `agent`, `provider`, `response_format`, `seed` stay inline in the
/// request log; the caller composes them with their own new `messages`
/// + the `continuation` string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestContinuation {
    /// The response_id of the original (about-to-be-continued)
    /// agent-completion turn. **Not necessarily the newest** request
    /// for the agent — `read_latest_continuation` walks back from
    /// newest to oldest and returns the first one whose continuation
    /// file exists.
    pub response_id: String,
    /// Continuation token to set as `continuation: Some(_)` on the
    /// fresh `AgentCompletionCreateParams`. Stored on-disk as a `.txt`
    /// file containing the raw base64 string under
    /// `logs/agents/completions/response/continuation/<response_id>.txt`.
    pub continuation: String,
    /// The agent definition from the original request log — reused
    /// verbatim so the conversation continues against the same agent.
    pub agent: crate::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional,
    /// Original provider routing preferences, if any.
    pub provider: Option<crate::agent::completions::request::Provider>,
    /// Original response format constraint, if any.
    pub response_format:
        Option<crate::agent::completions::request::ResponseFormatParam>,
    /// Original seed, if any. The caller decides whether to override
    /// with their own (e.g. cli `--seed`).
    pub seed: Option<i64>,
}

/// Three-way result of [`Client::read_latest_continuation`]. Lets the
/// caller distinguish "agent never ran" from "agent ran but no turn
/// has a continuation yet" without a follow-up query.
#[derive(Debug, Clone)]
pub enum LatestContinuationOutcome {
    /// Found a usable continuation. The `response_id` inside
    /// identifies the originating turn — not necessarily the newest
    /// one, because the walk-back skips turns whose continuation file
    /// is missing.
    Found(LatestContinuation),
    /// The agent_id has no `AgentCompletionRequest` rows at all —
    /// nothing has ever been logged against it.
    NoRequests,
    /// The agent_id has at least one request row but no continuation
    /// file was found in any of them — every prior turn is either
    /// still streaming or didn't finish. `request_count` is the total
    /// row count we walked.
    NoContinuationsFound {
        request_count: usize,
    },
}

impl Client {
    /// Walk the agent's request history newest-first and return the
    /// first turn whose continuation `.txt` exists, along with the
    /// inline `agent` / `provider` / `seed` extracted from that
    /// turn's request log. See [`LatestContinuationOutcome`] for the
    /// three terminal shapes.
    pub async fn read_latest_continuation(
        &self,
        agent_id: &str,
    ) -> Result<LatestContinuationOutcome, Error> {
        let response_ids = self
            .agent_completion_request_ids_newest_first(agent_id)
            .await?;
        if response_ids.is_empty() {
            return Ok(LatestContinuationOutcome::NoRequests);
        }
        let request_count = response_ids.len();

        let cont_dir = self
            .logs_dir()
            .join("agents/completions/response/continuation");

        for response_id in response_ids {
            let cont_path = cont_dir.join(format!("{response_id}.txt"));
            let continuation = match tokio::fs::read_to_string(&cont_path).await
            {
                Ok(s) => s,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(Error::Read(cont_path, e)),
            };

            // Found one — pair it with the per-turn request log.
            let request = self
                .read_agent_completion_request(&response_id, None)
                .await?;

            let agent: crate::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional =
                serde_json::from_value(
                    request.get("agent").cloned().ok_or_else(|| {
                        Error::InvalidPath(format!(
                            "agent_completion_request {response_id} missing `agent` field"
                        ))
                    })?,
                )
                .map_err(|e| {
                    Error::InvalidPath(format!(
                        "agent_completion_request {response_id} `agent` does not parse: {e}"
                    ))
                })?;

            let provider = match request.get("provider").cloned() {
                Some(serde_json::Value::Null) | None => None,
                Some(v) => Some(serde_json::from_value(v).map_err(|e| {
                    Error::InvalidPath(format!(
                        "agent_completion_request {response_id} `provider` does not parse: {e}"
                    ))
                })?),
            };

            // On-disk shape stores `response_format` as a
            // LogReference, not the inline ResponseFormatParam.
            // Resolving that indirection is a follow-up; for now
            // we let the new completion fall back to the agent's
            // default response format.
            let response_format = None;

            let seed = match request.get("seed").cloned() {
                Some(serde_json::Value::Number(n)) => n.as_i64(),
                _ => None,
            };

            return Ok(LatestContinuationOutcome::Found(LatestContinuation {
                response_id,
                continuation,
                agent,
                provider,
                response_format,
                seed,
            }));
        }

        Ok(LatestContinuationOutcome::NoContinuationsFound { request_count })
    }

    /// Every `AgentCompletionRequest.path` (which is the response_id)
    /// for `agent_id`, ordered by `"index"` descending — newest first.
    pub async fn agent_completion_request_ids_newest_first(
        &self,
        agent_id: &str,
    ) -> Result<Vec<String>, Error> {
        let conn = crate::filesystem::db::connection::connection(self)?;
        let agent_id = agent_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<String>, Error> {
            let conn = conn.lock().expect("filesystem db mutex poisoned");
            let mut stmt = conn.prepare_cached(
                "SELECT path FROM messages \
                 WHERE agent_id = ?1 AND kind = ?2 \
                 ORDER BY \"index\" DESC",
            )?;
            let rows = stmt.query_map(
                rusqlite::params![
                    agent_id,
                    crate::filesystem::db::schema::MessageKind::AgentCompletionRequest
                        .as_str()
                ],
                |r| r.get::<_, String>(0),
            )?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
        .await
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::Client;
    use crate::filesystem::db::schema::{MessageKind, init_tables, insert};

    /// Build a Client rooted at `base_dir`, insert N
    /// `AgentCompletionRequest` rows for `agent_id` with increasing
    /// indices, and write a minimal request-log JSON file for each so
    /// `read_latest_continuation`'s log fetch succeeds. Returns the
    /// list of response_ids in insertion order (oldest first).
    async fn seed_requests(
        base_dir: &std::path::Path,
        agent_id: &str,
        count: usize,
    ) -> Vec<String> {
        let client = Client::new(
            Some(base_dir.to_string_lossy().to_string()),
            None::<String>,
            None::<String>,
        );
        let conn = crate::filesystem::db::connection::connection(&client)
            .expect("open db");

        // Minimal valid request-log JSON: just an inline mock agent so
        // `serde_json::from_value::<InlineAgentBaseWithFallbacksOrRemoteCommitOptional>`
        // succeeds. The on-disk shape's Vec<LogReference> + extra
        // fields are tolerated as unknown keys when we only extract
        // `agent` / `provider` / `seed`.
        let request_log = serde_json::json!({
            "agent": { "remote": "mock", "name": "demo" },
            "messages": [],
            "stream": true
        });

        let requests_dir = client
            .logs_dir()
            .join("agents/completions/request");
        tokio::fs::create_dir_all(&requests_dir).await.expect("mkdir requests");

        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let response_id = format!("resp-{i:03}");
            // DB row.
            {
                let c = conn.lock().expect("conn lock");
                init_tables(&c).expect("init tables idempotent");
                insert(
                    &c,
                    agent_id,
                    MessageKind::AgentCompletionRequest,
                    &response_id,
                    1_700_000_000 + i as u64,
                    i as u64,
                )
                .expect("insert request row");
            }
            // Request log JSON.
            let req_path = requests_dir.join(format!("{response_id}.json"));
            tokio::fs::write(&req_path, request_log.to_string())
                .await
                .expect("write request log");
            ids.push(response_id);
        }
        ids
    }

    async fn write_continuation(
        base_dir: &std::path::Path,
        response_id: &str,
        body: &str,
    ) {
        let dir = base_dir
            .join("logs/agents/completions/response/continuation");
        tokio::fs::create_dir_all(&dir).await.expect("mkdir continuation");
        tokio::fs::write(dir.join(format!("{response_id}.txt")), body)
            .await
            .expect("write continuation");
    }

    #[tokio::test]
    async fn read_latest_continuation_walks_back_past_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let agent_id = "cli/foo";
        let ids = seed_requests(tmp.path(), agent_id, 3).await;
        // Only the OLDER turns have continuation files. The newest
        // (ids[2]) is "still streaming".
        write_continuation(tmp.path(), &ids[0], "cont-old").await;
        write_continuation(tmp.path(), &ids[1], "cont-mid").await;

        let client = Client::new(
            Some(tmp.path().to_string_lossy().to_string()),
            None::<String>,
            None::<String>,
        );
        let outcome = client
            .read_latest_continuation(agent_id)
            .await
            .expect("read_latest_continuation");
        match outcome {
            LatestContinuationOutcome::Found(c) => {
                // Should pick the NEWEST-with-continuation: ids[1],
                // not ids[2] (no file) and not ids[0] (older).
                assert_eq!(c.response_id, ids[1]);
                assert_eq!(c.continuation, "cont-mid");
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn read_latest_continuation_reports_no_continuations_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let agent_id = "cli/bar";
        let _ids = seed_requests(tmp.path(), agent_id, 2).await;
        // No continuation files at all.

        let client = Client::new(
            Some(tmp.path().to_string_lossy().to_string()),
            None::<String>,
            None::<String>,
        );
        let outcome = client
            .read_latest_continuation(agent_id)
            .await
            .expect("read_latest_continuation");
        match outcome {
            LatestContinuationOutcome::NoContinuationsFound { request_count } => {
                assert_eq!(request_count, 2);
            }
            other => panic!("expected NoContinuationsFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn read_latest_continuation_reports_no_requests() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let client = Client::new(
            Some(tmp.path().to_string_lossy().to_string()),
            None::<String>,
            None::<String>,
        );
        let outcome = client
            .read_latest_continuation("cli/never-existed")
            .await
            .expect("read_latest_continuation");
        match outcome {
            LatestContinuationOutcome::NoRequests => {}
            other => panic!("expected NoRequests, got {other:?}"),
        }
    }
}
