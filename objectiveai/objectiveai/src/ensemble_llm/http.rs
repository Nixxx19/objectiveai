use crate::{HttpClient, HttpError};

pub async fn list_ensemble_llms(
    client: &HttpClient,
) -> Result<Vec<super::response::ListEnsembleLlm>, HttpError> {
    client
        .send_unary(reqwest::Method::GET, "ensemble_llms", None::<String>)
        .await
}

pub async fn get_ensemble_llm(
    client: &HttpClient,
    ensemble_llm_id: &str,
) -> Result<super::response::GetEnsembleLlm, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("ensemble_llms/{}", ensemble_llm_id),
            None::<String>,
        )
        .await
}

pub async fn get_ensemble_llm_usage(
    client: &HttpClient,
    ensemble_llm_id: &str,
) -> Result<super::response::UsageEnsembleLlm, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("ensemble_llms/{}/usage", ensemble_llm_id),
            None::<String>,
        )
        .await
}
