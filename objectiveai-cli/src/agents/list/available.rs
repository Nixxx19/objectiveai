//! `agents list available <source>` — list remote agents from a
//! single source or the union of all sources.
//!
//! Thin dispatch over [`crate::list::Source`]: each variant delegates
//! to one of the shared list helpers in [`crate::list`]. Identical
//! behaviour to the pre-consolidation top-level `agents list <source>`
//! — only the parent path changed.

use objectiveai_sdk::cli::output::Handle;

use crate::list::Source;

pub async fn handle(
    source: Source,
    cli_config: &crate::Config,
    handle: &Handle,
) -> Result<(), crate::error::Error> {
    use objectiveai_sdk::agent::request::ListAgentsSource;
    match source {
        Source::Favorites => {
            crate::list::favorites(|| crate::agents::get_favorites(cli_config), handle).await
        }
        Source::Filesystem => {
            crate::list::single(
                cli_config,
                |c| Box::pin(list_source(c, ListAgentsSource::Filesystem)),
                handle,
            )
            .await
        }
        Source::Objectiveai => {
            crate::list::single(
                cli_config,
                |c| Box::pin(list_source(c, ListAgentsSource::Objectiveai)),
                handle,
            )
            .await
        }
        Source::Mock => {
            crate::list::single(
                cli_config,
                |c| Box::pin(list_source(c, ListAgentsSource::Mock)),
                handle,
            )
            .await
        }
        Source::All => {
            crate::list::all(
                cli_config,
                || crate::agents::get_favorites(cli_config),
                |c| Box::pin(list_source(c, ListAgentsSource::Filesystem)),
                |c| Box::pin(list_source(c, ListAgentsSource::Objectiveai)),
                handle,
            )
            .await
        }
    }
}

/// Fetch the list of remote agents from a single
/// [`ListAgentsSource`] via the SDK. Lives next to the only caller —
/// `agents/mod.rs` no longer needs it now that `list` is nested.
async fn list_source(
    http_client: objectiveai_sdk::HttpClient,
    source: objectiveai_sdk::agent::request::ListAgentsSource,
) -> Result<Vec<objectiveai_sdk::RemotePath>, crate::error::Error> {
    let response = objectiveai_sdk::agent::list_agents(
        &http_client,
        objectiveai_sdk::agent::request::ListAgentsRequest {
            source: Some(source),
        },
    )
    .await?;
    Ok(response.data)
}
