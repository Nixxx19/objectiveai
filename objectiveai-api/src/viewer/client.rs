use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::ctx;

pub struct Client<CTXEXT> {
    tx: mpsc::UnboundedSender<(ctx::Context<CTXEXT>, super::request::Request)>,
}

impl<CTXEXT: ctx::ContextExt + Send + Sync + 'static> Client<CTXEXT> {
    pub fn new(
        http_client: reqwest::Client,
        viewer_address: Option<String>,
        signature: Option<String>,
        backoff_current_interval: Duration,
        backoff_initial_interval: Duration,
        backoff_randomization_factor: f64,
        backoff_multiplier: f64,
        backoff_max_interval: Duration,
        backoff_max_elapsed_time: Duration,
    ) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<(ctx::Context<CTXEXT>, super::request::Request)>();

        let default_address = viewer_address.map(Arc::new);
        let default_signature = signature.map(Arc::new);

        tokio::spawn(async move {
            while let Some((ctx, request)) = rx.recv().await {
                // Resolve address and signature concurrently via select.
                // If ctx provides an address, use it with ctx signature.
                // If ctx has no address, use default address with default signature.
                // If both are None, skip.
                // If address resolves first to None, signature future is dropped.
                let addr_fut = ctx.objectiveai_viewer_address();
                let sig_fut = ctx.objectiveai_signature();
                tokio::pin!(addr_fut);
                tokio::pin!(sig_fut);

                let (address, signature) = tokio::select! {
                    biased;
                    addr = &mut addr_fut => {
                        match addr {
                            Some(addr) => {
                                let sig = sig_fut.await;
                                (addr, sig)
                            }
                            None => match &default_address {
                                Some(addr) => (addr.clone(), default_signature.clone()),
                                None => continue,
                            },
                        }
                    }
                    sig = &mut sig_fut => {
                        let addr = addr_fut.await;
                        match addr {
                            Some(addr) => (addr, sig),
                            None => match &default_address {
                                Some(addr) => (addr.clone(), default_signature.clone()),
                                None => continue,
                            },
                        }
                    }
                };

                let url = match &request {
                    super::request::Request::FunctionExecution(_) => {
                        format!("{}/functions/executions", address)
                    }
                    super::request::Request::FunctionInventionRecursive(_) => {
                        format!("{}/functions/inventions/recursive", address)
                    }
                };

                let body = match serde_json::to_vec(&request) {
                    Ok(body) => body,
                    Err(_) => continue,
                };

                let _ = backoff::future::retry(
                    backoff::ExponentialBackoff {
                        current_interval: backoff_current_interval,
                        initial_interval: backoff_initial_interval,
                        randomization_factor: backoff_randomization_factor,
                        multiplier: backoff_multiplier,
                        max_interval: backoff_max_interval,
                        max_elapsed_time: Some(backoff_max_elapsed_time),
                        start_time: std::time::Instant::now(),
                        clock: backoff::SystemClock::default(),
                    },
                    || {
                        let http_client = &http_client;
                        let url = &url;
                        let body = &body;
                        let signature = &signature;
                        async move {
                            let mut req = http_client
                                .post(url.as_str())
                                .header("Content-Type", "application/json")
                                .body(body.clone());

                            if let Some(sig) = signature {
                                req = req.header("X-OBJECTIVEAI-SIGNATURE", sig.as_str());
                            }

                            let response = req.send().await
                                .map_err(backoff::Error::transient)?;

                            if response.status().is_success() {
                                Ok(())
                            } else {
                                Err(backoff::Error::transient(
                                    response.error_for_status().unwrap_err()
                                ))
                            }
                        }
                    },
                ).await;
            }
        });

        Self { tx }
    }

    pub fn send_function_execution_begin(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: String,
        request: objectiveai::functions::executions::request::FunctionExecutionCreateParams,
    ) {
        self.tx.send((ctx, super::request::Request::FunctionExecution(
            super::request::FunctionExecutionRequest::Begin(super::request::FunctionExecutionCreateParams {
                id,
                inner: request,
            }),
        ))).ok();
    }

    pub fn send_function_execution_continue(
        &self,
        ctx: ctx::Context<CTXEXT>,
        chunk: objectiveai::functions::executions::response::streaming::FunctionExecutionChunk,
    ) {
        self.tx.send((ctx, super::request::Request::FunctionExecution(
            super::request::FunctionExecutionRequest::Continue(chunk),
        ))).ok();
    }

    pub fn send_function_execution_error(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: String,
        error: objectiveai::error::ResponseError,
    ) {
        self.tx.send((ctx, super::request::Request::FunctionExecution(
            super::request::FunctionExecutionRequest::Error(super::request::ResponseError {
                id,
                inner: error,
            }),
        ))).ok();
    }

    pub fn send_function_invention_recursive_begin(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: String,
        request: objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams,
    ) {
        self.tx.send((ctx, super::request::Request::FunctionInventionRecursive(
            super::request::FunctionInventionRecursiveRequest::Begin(super::request::FunctionInventionRecursiveCreateParams {
                id,
                inner: request,
            }),
        ))).ok();
    }

    pub fn send_function_invention_recursive_continue(
        &self,
        ctx: ctx::Context<CTXEXT>,
        chunk: objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk,
    ) {
        self.tx.send((ctx, super::request::Request::FunctionInventionRecursive(
            super::request::FunctionInventionRecursiveRequest::Continue(chunk),
        ))).ok();
    }

    pub fn send_function_invention_recursive_error(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: String,
        error: objectiveai::error::ResponseError,
    ) {
        self.tx.send((ctx, super::request::Request::FunctionInventionRecursive(
            super::request::FunctionInventionRecursiveRequest::Error(super::request::ResponseError {
                id,
                inner: error,
            }),
        ))).ok();
    }
}
