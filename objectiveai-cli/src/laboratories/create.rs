use futures::StreamExt;

use super::create_args::CreateArgs;

pub async fn handle(args: CreateArgs) -> Result<crate::Output, crate::error::Error> {
    let builder_agents = args
        .builder_agent
        .into_iter()
        .map(|a| a.resolve())
        .collect::<Result<Vec<_>, _>>()?;
    let evaluation_agent = args.evaluation_agent.resolve()?;
    let builder_messages = args.builder_messages.resolve()?;
    let evaluation_messages = args.evaluation_messages.resolve()?;
    let evaluation_output_schema = args.evaluation_output_schema.resolve()?;
    let builder_continuation = args.builder_continuation.resolve()?;
    let evaluation_continuation = args.evaluation_continuation.resolve()?;

    let params = objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams {
        docker_image: args.docker_image,
        builder_agents,
        evaluation_agent,
        builder_messages,
        evaluation_messages,
        evaluation_output_schema,
        builder_continuation,
        evaluation_continuation,
        max_evaluation_retries: args.max_evaluation_retries,
        provider: None,
        seed: args.seed,
        stream: Some(true),
    };

    crate::api::run(|http_client| async move {
        let stream = objectiveai::laboratories::executions::create_laboratory_execution_streaming(
            &http_client, params,
        ).await?;
        tokio::pin!(stream);

        let mut accumulated: Option<
            objectiveai::laboratories::executions::response::streaming::LaboratoryExecutionChunk,
        > = None;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            match &mut accumulated {
                Some(agg) => agg.push(&chunk),
                None => accumulated = Some(chunk),
            }
        }

        let _execution: objectiveai::laboratories::executions::response::unary::LaboratoryExecution =
            accumulated.ok_or(crate::error::Error::EmptyStream)?.into();

        unimplemented!()
    }, true).await
}
