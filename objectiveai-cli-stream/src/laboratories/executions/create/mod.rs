//! `laboratories executions create` — open a laboratory execution
//! stream, emit each chunk as NDJSON to stdout, manage per-agent
//! named pipes, and write coalesced log files to
//! `${config_base_dir}/logs/`.

use objectiveai_sdk::cli::output::Handle;
use objectiveai_sdk::laboratories::executions::request::LaboratoryExecutionCreateParams;
use objectiveai_sdk::laboratories::executions::response::streaming::LaboratoryExecutionChunk;

use crate::api::{BodySource, HttpArgs, PipeArgs};
use crate::streaming;

pub async fn handle(
    http: &HttpArgs,
    pipes: &PipeArgs,
    body: BodySource,
    handle: &Handle,
) -> Result<(), String> {
    let params: LaboratoryExecutionCreateParams = body.resolve()?;
    let config_base_dir = pipes.config_base_dir()?.to_path_buf();
    let pipes_root = pipes.pipes_root()?;
    let client = http.build_http_client()?;
    let conduit = pipes.build_conduit();

    let fs_client = objectiveai_sdk::filesystem::Client::new(
        Some(config_base_dir),
        None::<String>,
        None::<String>,
    );
    let log_writer = fs_client
        .write_laboratory_execution(&params)
        .map_err(|e| format!("failed to build laboratory-execution log writer: {e}"))?;

    let (stream, notifier) =
        objectiveai_sdk::laboratories::executions::create_laboratory_execution_streaming(
            &client, params, conduit,
        )
        .await
        .map_err(|e| format!("failed to open laboratory-execution stream: {e}"))?;

    let stream = Box::pin(stream);

    let consumed = streaming::run_chunk_loop::<_, LaboratoryExecutionChunk, _, _>(
        stream,
        notifier,
        pipes_root,
        log_writer,
        handle,
        |agg: &mut LaboratoryExecutionChunk, chunk: &LaboratoryExecutionChunk| agg.push(chunk),
    )
    .await?;

    if let Some(error) = consumed.aggregate.and_then(|a| a.error) {
        return Err(format!("laboratory execution failed: {error:?}"));
    }
    Ok(())
}
