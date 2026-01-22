use crate::{ctx, util::StreamOnce};
use futures::{Stream, StreamExt, TryStreamExt};
use std::{sync::Arc, time::Duration};

#[derive(Debug, Clone)]
pub struct Client {
    pub openrouter_client: super::openrouter::Client,
    // pub google_vertex_client: super::google_vertex::Client,
}

impl Client {
    pub fn new(openrouter_client: super::openrouter::Client) -> Self {
        Self { openrouter_client }
    }

    pub async fn create_streaming(
        &self,
        ctx: ctx::Context<impl ctx::ContextExt + Send + Sync + 'static>,
        id: String,
        first_chunk_timeout: Duration,
        other_chunk_timeout: Duration,
        ensemble_llm: Arc<objectiveai::ensemble_llm::EnsembleLlm>,
        request: super::Params,
    ) -> Result<
        Option<
            impl Stream<
                Item = Result<
                    objectiveai::chat::completions::response::streaming::ChatCompletionChunk,
                    super::Error,
                >,
            > + Send
            + Unpin
            + 'static,
        >,
        super::Error,
    >{
        let mut errors = Vec::new();

        // try each upstream in order
        for upstream in super::upstreams(&ensemble_llm, request.clone()) {
            // fetch BYOK from context
            let byok = ctx
                .ext
                .get_byok(upstream)
                .await
                .map_err(super::Error::FetchByok)?;

            // first, try BYOK if available
            if let Some(byok) = byok {
                match self
                    .upstream_create_streaming(
                        upstream,
                        id.clone(),
                        Some(byok),
                        ctx.cost_multiplier,
                        first_chunk_timeout,
                        other_chunk_timeout,
                        ensemble_llm.clone(),
                        request.clone(),
                    )
                    .await
                {
                    Ok(stream) => {
                        return Ok(Some(stream));
                    }
                    Err(e) => {
                        errors.push(e);
                    }
                }
            }

            // then, try without BYOK
            match self
                .upstream_create_streaming(
                    upstream,
                    id.clone(),
                    None,
                    ctx.cost_multiplier,
                    first_chunk_timeout,
                    other_chunk_timeout,
                    ensemble_llm.clone(),
                    request.clone(),
                )
                .await
            {
                Ok(stream) => {
                    return Ok(Some(stream));
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(None)
        } else {
            Err(super::Error::MultipleErrors(errors))
        }
    }

    async fn upstream_create_streaming(
        &self,
        upstream: super::Upstream,
        id: String,
        byok: Option<String>,
        cost_multiplier: rust_decimal::Decimal,
        first_chunk_timeout: Duration,
        other_chunk_timeout: Duration,
        ensemble_llm: Arc<objectiveai::ensemble_llm::EnsembleLlm>,
        request: super::Params,
    ) -> Result<
        impl Stream<
            Item = Result<
                objectiveai::chat::completions::response::streaming::ChatCompletionChunk,
                super::Error,
            >,
        > + Send
        + Unpin
        + 'static,
        super::Error,
    >{
        let mut stream = match request {
            super::Params::Chat { request } => self
                .create_streaming_for_chat(
                    upstream,
                    id,
                    byok.as_deref(),
                    cost_multiplier,
                    first_chunk_timeout,
                    other_chunk_timeout,
                    &ensemble_llm,
                    &request,
                )
                .boxed(),
            super::Params::Vector {
                request,
                vector_pfx_indices,
            } => self
                .create_streaming_for_vector(
                    upstream,
                    id,
                    byok.as_deref(),
                    cost_multiplier,
                    first_chunk_timeout,
                    other_chunk_timeout,
                    &ensemble_llm,
                    &request,
                    &vector_pfx_indices,
                )
                .boxed(),
        };
        match stream.try_next().await {
            Ok(Some(chunk)) => Ok(StreamOnce::new(Ok(chunk)).chain(stream)),
            Ok(None) => Err(super::Error::EmptyStream),
            Err(e) => Err(e),
        }
    }

    fn create_streaming_for_chat(
        &self,
        upstream: super::Upstream,
        id: String,
        byok: Option<&str>,
        cost_multiplier: rust_decimal::Decimal,
        first_chunk_timeout: Duration,
        other_chunk_timeout: Duration,
        ensemble_llm: &objectiveai::ensemble_llm::EnsembleLlm,
        request: &objectiveai::chat::completions::request::ChatCompletionCreateParams,
    ) -> impl Stream<
        Item = Result<
            objectiveai::chat::completions::response::streaming::ChatCompletionChunk,
            super::Error,
        >,
    > + Send
    + 'static{
        match upstream {
            super::Upstream::OpenRouter => self
                .openrouter_client
                .create_streaming_for_chat(
                    id,
                    byok,
                    cost_multiplier,
                    first_chunk_timeout,
                    other_chunk_timeout,
                    ensemble_llm,
                    request,
                )
                .map_err(super::Error::from),
        }
    }

    fn create_streaming_for_vector(
        &self,
        upstream: super::Upstream,
        id: String,
        byok: Option<&str>,
        cost_multiplier: rust_decimal::Decimal,
        first_chunk_timeout: Duration,
        other_chunk_timeout: Duration,
        ensemble_llm: &objectiveai::ensemble_llm::EnsembleLlm,
        request: &objectiveai::vector::completions::request::VectorCompletionCreateParams,
        vector_pfx_indices: &[(String, usize)],
    ) -> impl Stream<
        Item = Result<
            objectiveai::chat::completions::response::streaming::ChatCompletionChunk,
            super::Error,
        >,
    > + Send
    + 'static{
        match upstream {
            super::Upstream::OpenRouter => self
                .openrouter_client
                .create_streaming_for_vector(
                    id,
                    byok,
                    cost_multiplier,
                    first_chunk_timeout,
                    other_chunk_timeout,
                    ensemble_llm,
                    request,
                    vector_pfx_indices,
                )
                .map_err(super::Error::from),
        }
    }
}
