//! Mock Function fetcher that loads from embedded JSON assets.

use crate::ctx;

/// Mock Function fetcher for testing.
///
/// Expects owner `"mock"` and commit `"mock"`. Matches repository name
/// to embedded JSON fixtures (e.g. `"mock-1"` → `mock-1.json`).
pub struct MockFetcher;

#[async_trait::async_trait]
impl<CTXEXT> super::super::Fetcher<CTXEXT> for MockFetcher
where
    CTXEXT: Send + Sync + 'static,
{
    async fn fetch(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        Option<super::super::FullGetFunction>,
        objectiveai::error::ResponseError,
    > {
        if owner != "mock" || commit != Some("mock") {
            return Ok(None);
        }
        let json = match repository {
            "mock-1" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-1.json")),
            "mock-2" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-2.json")),
            "mock-3" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-3.json")),
            "mock-4" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-4.json")),
            "mock-5" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-5.json")),
            "mock-6" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-6.json")),
            "mock-7" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-7.json")),
            "mock-8" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-8.json")),
            "mock-9" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-9.json")),
            "mock-10" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-10.json")),
            "mock-11" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-11.json")),
            "mock-12" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-12.json")),
            "mock-13" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-13.json")),
            "mock-14" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-14.json")),
            "mock-15" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-15.json")),
            "mock-16" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-16.json")),
            "mock-17" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-17.json")),
            _ => return Ok(None),
        };
        let inner: objectiveai::functions::FullRemoteFunction =
            serde_json::from_str(json).expect("invalid mock function JSON");
        Ok(Some(super::super::FullGetFunction {
            remote: objectiveai::functions::Remote::Mock,
            owner: owner.to_string(),
            repository: repository.to_string(),
            commit: "mock".to_string(),
            inner,
        }))
    }
}
