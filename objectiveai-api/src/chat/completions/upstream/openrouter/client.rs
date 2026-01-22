use eventsource_stream::Event as MessageEvent;
use futures::{Stream, StreamExt};
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use std::time::Duration;

pub fn response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("chtcpl-{}-{}", uuid.simple(), created)
}

#[derive(Debug, Clone)]
pub struct Client {
    pub http_client: reqwest::Client,
    pub api_base: String,
    pub api_key: String,
    pub user_agent: Option<String>, // user-agent header
    pub x_title: Option<String>,    // x-title header
    pub referer: Option<String>,    // referer and http-referer headers
}

impl Client {
    pub fn new(
        http_client: reqwest::Client,
        api_base: String,
        api_key: String,
        user_agent: Option<String>,
        x_title: Option<String>,
        referer: Option<String>,
    ) -> Self {
        Self {
            http_client,
            api_base,
            api_key,
            user_agent,
            x_title,
            referer,
        }
    }

    pub fn create_streaming_for_chat(
        &self,
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
    + 'static {
        self.create_streaming(
            id,
            ensemble_llm.id.clone(),
            byok,
            cost_multiplier,
            first_chunk_timeout,
            other_chunk_timeout,
            super::request::ChatCompletionCreateParams::new_for_chat(ensemble_llm, request),
        )
    }

    pub fn create_streaming_for_vector(
        &self,
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
    + 'static {
        self.create_streaming(
            id,
            ensemble_llm.id.clone(),
            byok,
            cost_multiplier,
            first_chunk_timeout,
            other_chunk_timeout,
            super::request::ChatCompletionCreateParams::new_for_vector(
                vector_pfx_indices,
                ensemble_llm,
                request,
            ),
        )
    }

    fn create_streaming(
        &self,
        id: String,
        model: String,
        byok: Option<&str>,
        cost_multiplier: rust_decimal::Decimal,
        first_chunk_timeout: Duration,
        other_chunk_timeout: Duration,
        request: super::request::ChatCompletionCreateParams,
    ) -> impl Stream<
        Item = Result<
            objectiveai::chat::completions::response::streaming::ChatCompletionChunk,
            super::Error,
        >,
    > + Send
    + 'static {
        let is_byok = byok.is_some();
        let event_source =
            self.create_streaming_event_source(byok.unwrap_or(&self.api_key), &request);
        Self::create_streaming_stream(
            event_source,
            id,
            model,
            is_byok,
            cost_multiplier,
            first_chunk_timeout,
            other_chunk_timeout,
        )
    }

    fn create_streaming_event_source(
        &self,
        api_key: &str,
        request: &super::request::ChatCompletionCreateParams,
    ) -> EventSource {
        let mut http_request = self
            .http_client
            .post(format!("{}/chat/completions", self.api_base))
            .header("authorization", format!("Bearer {}", api_key));
        if let Some(ref user_agent) = self.user_agent {
            http_request = http_request.header("user-agent", user_agent);
        }
        if let Some(ref x_title) = self.x_title {
            http_request = http_request.header("x-title", x_title);
        }
        if let Some(ref referer) = self.referer {
            http_request = http_request
                .header("referer", referer)
                .header("http-referer", referer);
        }
        http_request.json(request).eventsource().unwrap()
    }

    fn create_streaming_stream(
        mut event_source: EventSource,
        id: String,
        model: String,
        is_byok: bool,
        cost_multiplier: rust_decimal::Decimal,
        first_chunk_timeout: Duration,
        other_chunk_timeout: Duration,
    ) -> impl Stream<
        Item = Result<
            objectiveai::chat::completions::response::streaming::ChatCompletionChunk,
            super::Error,
        >,
    > + Send
    + 'static {
        async_stream::stream! {
            let mut first = true;
            while let Some(event) = tokio::time::timeout(
                if first {
                    first_chunk_timeout
                } else {
                    other_chunk_timeout
                },
                event_source.next(),
            ).await.transpose() {
                first = false;
                match event {
                    Ok(Ok(Event::Open)) => continue,
                    Ok(Ok(Event::Message(MessageEvent { data, .. }))) => {
                        if data == "[DONE]" {
                            break;
                        } else if data.starts_with(":") {
                            continue; // skip comments
                        } else if data.is_empty() {
                            continue; // skip empty messages
                        }
                        let mut de = serde_json::Deserializer::from_str(&data);
                        match serde_path_to_error::deserialize::<
                            _,
                            super::response::ChatCompletionChunk,
                        >(&mut de)
                        {
                            Ok(chunk) => yield Ok(chunk.into_downstream(
                                id.clone(),
                                model.clone(),
                                is_byok,
                                cost_multiplier,
                            )),
                            Err(e) => {
                                de = serde_json::Deserializer::from_str(&data);
                                match serde_path_to_error::deserialize::<
                                    _,
                                    super::OpenRouterProviderError,
                                >(&mut de)
                                {
                                    Ok(provider_error) => yield Err(
                                        super::Error::OpenRouterProviderError(
                                            provider_error,
                                        ),
                                    ),
                                    Err(_) => yield Err(
                                        super::Error::DeserializationError(e),
                                    ),
                                }
                            }
                        }
                    }
                    Ok(Err(reqwest_eventsource::Error::InvalidStatusCode(
                        code,
                        response,
                    ))) => {
                        match response.text().await {
                            Ok(body) => {
                                yield Err(super::Error::BadStatus {
                                    code,
                                    body: match serde_json::from_str::<
                                        serde_json::Value,
                                    >(
                                        &body,
                                    ) {
                                        Ok(value) => value,
                                        Err(_) => serde_json::Value::String(
                                            body,
                                        ),
                                    },
                                });
                            }
                            Err(_) => {
                                yield Err(super::Error::BadStatus {
                                    code,
                                    body: serde_json::Value::Null,
                                });
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        yield Err(super::Error::from(e));
                    }
                    Err(_) => {
                        yield Err(super::Error::StreamTimeout);
                    }
                }
            }
        }
    }
}
