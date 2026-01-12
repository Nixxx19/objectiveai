use crate::error;
use eventsource_stream::Event as MessageEvent;
use futures::{Stream, StreamExt};
use reqwest_eventsource::{Event, RequestBuilderExt};

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub http_client: reqwest::Client,
    pub api_base: String,
    pub api_key: Option<String>,
    pub user_agent: Option<String>, // user-agent header
    pub x_title: Option<String>,    // x-title header
    pub referer: Option<String>,    // referer and http-referer headers
}

impl HttpClient {
    pub fn new(
        http_client: reqwest::Client,
        api_base: Option<impl Into<String>>,
        api_key: Option<impl Into<String>>,
        user_agent: Option<impl Into<String>>,
        x_title: Option<impl Into<String>>,
        referer: Option<impl Into<String>>,
    ) -> Self {
        Self {
            http_client,
            api_base: match api_base {
                Some(base) => base.into(),
                None => "https://api.objective-ai.io".to_string(),
            },
            api_key: api_key.map(Into::into),
            user_agent: user_agent.map(Into::into),
            x_title: x_title.map(Into::into),
            referer: referer.map(Into::into),
        }
    }

    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<impl serde::Serialize>,
    ) -> reqwest::RequestBuilder {
        let url = format!(
            "{}/{}",
            self.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut request = self.http_client.request(method, &url);
        if let Some(api_key) = &self.api_key {
            request =
                request.header("authorization", format!("Bearer {}", api_key));
        }
        if let Some(user_agent) = &self.user_agent {
            request = request.header("user-agent", user_agent);
        }
        if let Some(x_title) = &self.x_title {
            request = request.header("x-title", x_title);
        }
        if let Some(referer) = &self.referer {
            request = request.header("referer", referer);
            request = request.header("http-referer", referer);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        request
    }

    pub async fn send_unary<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
        body: Option<impl serde::Serialize>,
    ) -> Result<T, super::HttpError> {
        let response = self
            .http_client
            .execute(
                self.request(method, path.as_ref(), body)
                    .build()
                    .map_err(super::HttpError::RequestError)?,
            )
            .await
            .map_err(super::HttpError::HttpError)?;
        let code = response.status();
        if code.is_success() {
            let text =
                response.text().await.map_err(super::HttpError::HttpError)?;
            let mut de = serde_json::Deserializer::from_str(&text);
            match serde_path_to_error::deserialize::<_, T>(&mut de) {
                Ok(value) => Ok(value),
                Err(e) => Err(super::HttpError::DeserializationError(e)),
            }
        } else {
            match response.text().await {
                Ok(text) => Err(super::HttpError::BadStatus {
                    code,
                    body: match serde_json::from_str::<serde_json::Value>(&text)
                    {
                        Ok(body) => body,
                        Err(_) => serde_json::Value::String(text),
                    },
                }),
                Err(_) => Err(super::HttpError::BadStatus {
                    code,
                    body: serde_json::Value::Null,
                }),
            }
        }
    }

    pub async fn send_unary_no_response(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
        body: Option<impl serde::Serialize>,
    ) -> Result<(), super::HttpError> {
        let response = self
            .http_client
            .execute(
                self.request(method, path.as_ref(), body)
                    .build()
                    .map_err(super::HttpError::RequestError)?,
            )
            .await
            .map_err(super::HttpError::HttpError)?;
        let code = response.status();
        if code.is_success() {
            Ok(())
        } else {
            match response.text().await {
                Ok(text) => Err(super::HttpError::BadStatus {
                    code,
                    body: match serde_json::from_str::<serde_json::Value>(&text)
                    {
                        Ok(body) => body,
                        Err(_) => serde_json::Value::String(text),
                    },
                }),
                Err(_) => Err(super::HttpError::BadStatus {
                    code,
                    body: serde_json::Value::Null,
                }),
            }
        }
    }

    pub async fn send_streaming<
        T: serde::de::DeserializeOwned + Send + 'static,
    >(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
        body: Option<impl serde::Serialize>,
    ) -> Result<
        impl Stream<Item = Result<T, super::HttpError>> + Send + 'static,
        super::HttpError,
    > {
        Ok(
            self.request(method, path.as_ref(), body)
                .eventsource()?
                .then(|result| async {
                    match result {
                        Ok(Event::Open) => None,
                        Ok(Event::Message(MessageEvent { data, .. }))
                            if data == "[DONE]"
                                || data.starts_with(":")
                                || data.is_empty() =>
                        {
                            None
                        }
                        Ok(Event::Message(MessageEvent { data, .. })) => {
                            let mut de =
                                serde_json::Deserializer::from_str(&data);
                            Some(
                                match serde_path_to_error::deserialize::<_, T>(
                                    &mut de,
                                ) {
                                    Ok(value) => Ok(value),
                                    Err(e) => match serde_json::from_str::<error::ResponseError>(&data) {
                                        Ok(err) => Err(super::HttpError::ApiError(err)),
                                        Err(_) => Err(super::HttpError::DeserializationError(e)),
                                    },
                                }
                            )
                        }
                        Err(reqwest_eventsource::Error::InvalidStatusCode(
                            code,
                            response,
                        )) => match response.text().await {
                            Ok(body) => {
                                Some(Err(super::HttpError::BadStatus {
                                    code,
                                    body: match serde_json::from_str::<
                                        serde_json::Value,
                                    >(
                                        &body
                                    ) {
                                        Ok(body) => body,
                                        Err(_) => {
                                            serde_json::Value::String(body)
                                        }
                                    },
                                }))
                            }
                            Err(_) => Some(Err(super::HttpError::BadStatus {
                                code,
                                body: serde_json::Value::Null,
                            })),
                        },
                        Err(e) => Some(Err(super::HttpError::StreamError(e))),
                    }
                })
                .filter_map(|x| async { x }),
        )
    }
}
