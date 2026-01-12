use crate::{HttpClient, HttpError};

pub async fn list_ensembles(
    client: &HttpClient,
) -> Result<Vec<super::response::ListEnsemble>, HttpError> {
    client
        .send_unary(reqwest::Method::GET, "ensembles", None::<String>)
        .await
}

pub async fn get_ensemble(
    client: &HttpClient,
    ensemble_id: &str,
) -> Result<super::response::GetEnsemble, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("ensembles/{}", ensemble_id),
            None::<String>,
        )
        .await
}

pub async fn get_ensemble_usage(
    client: &HttpClient,
    ensemble_id: &str,
) -> Result<super::response::UsageEnsemble, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("ensembles/{}/usage", ensemble_id),
            None::<String>,
        )
        .await
}
