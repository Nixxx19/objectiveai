//! `read_latest_continuation` — given an `agent_id`, find the most
//! recent `AgentCompletionRequest` row in the messages DB, read its
//! request log + continuation file, and return enough state to POST a
//! fresh agent-completion that resumes the conversation.
//!
//! Used by `objectiveai agents message`'s fallback path: if the live
//! per-agent socket is unreachable, we resume the agent's last turn
//! via continuation instead of dropping the message on the floor.

use serde::{Deserialize, Serialize};

use crate::filesystem::{Client, Error};

/// Everything needed to POST a continuation that re-enters an agent's
/// most recent conversation.
///
/// Field-set mirrors the inline-on-disk shape of
/// [`crate::agent::completions::request::AgentCompletionCreateParamsLog`]:
/// `agent`, `provider`, `response_format`, `seed` stay inline in the
/// request log; the caller composes them with their own new `messages`
/// + the `continuation` string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestContinuation {
    /// The response_id of the original (about-to-be-continued)
    /// agent-completion turn. Also the filename stem of every per-id
    /// log file (`<response_id>.json`).
    pub response_id: String,
    /// Continuation token to set as `continuation: Some(_)` on the
    /// fresh `AgentCompletionCreateParams`. Stored on-disk as a JSON
    /// string under
    /// `logs/agents/completions/response/continuation/<response_id>.json`.
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

impl Client {
    /// Look up the most recent `AgentCompletionRequest` for
    /// `agent_id` in the messages DB and assemble its
    /// [`LatestContinuation`]. Returns `Ok(None)` if either no
    /// prior request exists OR the most-recent request has no
    /// continuation file (e.g. the response hasn't finished yet).
    /// Both cases are non-error: the caller (`agents message`)
    /// distinguishes them by message and exits non-zero in either
    /// situation.
    pub async fn read_latest_continuation(
        &self,
        agent_id: &str,
    ) -> Result<Option<LatestContinuation>, Error> {
        let response_id = match self
            .latest_agent_completion_request_id(agent_id)
            .await?
        {
            Some(id) => id,
            None => return Ok(None),
        };

        // Request log is mandatory if we found a request row.
        let request = self
            .read_agent_completion_request(&response_id, None)
            .await?;

        // Continuation file is optional (in-progress turns don't have
        // one yet); treat NotFound as None. Stored as a `.txt` file
        // containing the raw base64 continuation string — see
        // `AgentCompletionCreateParams::produce_files` step 3.
        let cont_path = self
            .logs_dir()
            .join("agents/completions/response/continuation")
            .join(format!("{response_id}.txt"));
        let continuation = match tokio::fs::read_to_string(&cont_path).await {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(e) => return Err(Error::Read(cont_path, e)),
        };

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

        let response_format = match request.get("response_format").cloned() {
            Some(serde_json::Value::Null) | None => None,
            // The on-disk shape stores `response_format` as a LogReference,
            // not the inline ResponseFormatParam — defer this to a
            // follow-up if the caller actually needs it. For now,
            // return None so the new completion uses the agent's default.
            Some(_) => None,
        };

        let seed = match request.get("seed").cloned() {
            Some(serde_json::Value::Number(n)) => n.as_i64(),
            _ => None,
        };

        Ok(Some(LatestContinuation {
            response_id,
            continuation,
            agent,
            provider,
            response_format,
            seed,
        }))
    }

    /// `SELECT path FROM messages WHERE agent_id = ? AND kind =
    /// 'agent_completion_request' ORDER BY "index" DESC LIMIT 1`.
    /// For `AgentCompletionRequest` rows, `path` is the response_id.
    pub async fn latest_agent_completion_request_id(
        &self,
        agent_id: &str,
    ) -> Result<Option<String>, Error> {
        let conn = crate::filesystem::db::connection::connection(self)?;
        let agent_id = agent_id.to_string();
        tokio::task::spawn_blocking(move || -> Result<Option<String>, Error> {
            let conn = conn
                .lock()
                .expect("filesystem db mutex poisoned");
            let mut stmt = conn.prepare_cached(
                "SELECT path FROM messages \
                 WHERE agent_id = ?1 AND kind = ?2 \
                 ORDER BY \"index\" DESC LIMIT 1",
            )?;
            use rusqlite::OptionalExtension as _;
            let row = stmt
                .query_row(
                    rusqlite::params![
                        agent_id,
                        crate::filesystem::db::schema::MessageKind::AgentCompletionRequest
                            .as_str()
                    ],
                    |r| r.get::<_, String>(0),
                )
                .optional()?;
            Ok(row)
        })
        .await
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
}
