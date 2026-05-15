//! Tauri command `cli_run` — drives `objectiveai_cli::run()` in-process
//! and pipes its JSONL output back to the iframe that invoked it as
//! a stream of [`Event::CliCommand`] events.
//!
//! The plugin-bridge resolves the originating iframe (via
//! `MessageEvent.source`) and passes its repository name as `origin`,
//! which becomes the `destination` on every emitted event. The plugin
//! never sets `destination` itself.

use objectiveai_sdk::cli::output::Handle;
use tokio::sync::mpsc;

use crate::events::{Event, EventSender};

/// Drive the cli with `args` in-process. Each JSONL line the cli
/// emits is wrapped as `Event::CliCommand { destination: origin, value }`
/// and pushed onto the viewer's events bus, where the JS bridge picks
/// it up and forwards to the originating iframe.
///
/// Returns immediately after spawning the cli + forwarder tasks; the
/// iframe sees output asynchronously via the events channel. When the
/// cli's `run()` completes, the `Handle::Stream` sender is dropped,
/// the forwarder loop exits, and any handler waiting on the stream
/// terminates naturally on the `{"type":"end"}` line.
#[tauri::command]
pub async fn cli_run(
    events_tx: tauri::State<'_, EventSender>,
    args: Vec<String>,
    origin: String,
) -> Result<(), String> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let handle = Handle::Stream(tx);
    let events_tx_clone = events_tx.inner().clone();
    let origin_for_forwarder = origin.clone();

    // Forwarder: read each cli Output, wrap as Event::CliCommand,
    // push to the events bus. Loop exits when the handle is dropped.
    tokio::spawn(async move {
        while let Some(output) = rx.recv().await {
            let value = serde_json::to_value(&output).unwrap_or(serde_json::Value::Null);
            let _ = events_tx_clone.send(Event::CliCommand {
                destination: origin_for_forwarder.clone(),
                value,
            });
        }
    });

    // Spawn the cli run. cli's run() consumes the Handle and emits to
    // the channel until it finishes; then the Handle drops, the
    // forwarder's loop ends.
    let cli_config = objectiveai_cli::load_config();
    tokio::spawn(async move {
        let _exit_code = objectiveai_cli::run(args, &cli_config, handle).await;
    });

    Ok(())
}
