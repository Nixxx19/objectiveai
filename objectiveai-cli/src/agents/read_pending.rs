//! `agents read_pending` — drain every unread queue row for the
//! given spawned agent id(s) from this CLI's perspective.
//!
//! Wraps [`objectiveai_sdk::filesystem::Client::read_new_from_queue`]:
//! atomically advances the per-(caller, spawned) watermark in
//! `messages_queue` and returns each unread row as a typed
//! `QueueItem`. The caller id is taken from `cli_config.agent_id`
//! (env-supplied `OBJECTIVEAI_AGENT_ID`, defaulting to `"cli"` in
//! `main.rs`).
//!
//! Accepts one or more spawned agent ids as positional args; each
//! is drained in order and the resulting items are concatenated into
//! a single `Items<QueueItem>` notification.

use clap::Args;
use objectiveai_sdk::cli::output::{Handle, Items, Notification, Output};
use objectiveai_sdk::filesystem::logs::queue::QueueItem;

#[derive(Args)]
pub struct CommandArgs {
    /// Spawned agent ids to drain. Repeat the positional argument for
    /// multiple agents — e.g.
    /// `agents read_pending agent-a agent-b`.
    #[arg(required = true)]
    pub agent_ids: Vec<String>,
}

pub async fn handle(
    args: CommandArgs,
    cli_config: &crate::Config,
    handle: &Handle,
) -> Result<(), crate::error::Error> {
    let client = objectiveai_sdk::filesystem::Client::new(
        cli_config.config_base_dir.as_deref(),
        None::<String>,
        None::<String>,
    );
    let caller = cli_config.agent_id.as_deref().unwrap_or("cli");

    let mut items: Vec<QueueItem> = Vec::new();
    for spawned in &args.agent_ids {
        let batch = client.read_new_from_queue(caller, spawned).await?;
        items.extend(batch);
    }

    Output::<Items<QueueItem>>::Notification(Notification {
        agent_id: None,
        value: Items { items },
    })
    .emit(handle)
    .await;
    Ok(())
}
