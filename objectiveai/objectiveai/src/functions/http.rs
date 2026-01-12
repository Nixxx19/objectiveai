use crate::{HttpClient, HttpError};

pub async fn list_functions(
    client: &HttpClient,
) -> Result<Vec<super::response::ListFunction>, HttpError> {
    client
        .send_unary(reqwest::Method::GET, "functions", None::<String>)
        .await
}

pub async fn get_function_usage(
    client: &HttpClient,
    fowner: &str,
    frepository: &str,
    fcommit: Option<&str>,
) -> Result<super::response::UsageFunction, HttpError> {
    let path = match fcommit {
        Some(fcommit) => {
            format!("functions/{}/{}/{}/usage", fowner, frepository, fcommit)
        }
        None => format!("functions/{}/{}/usage", fowner, frepository),
    };
    client
        .send_unary(reqwest::Method::GET, &path, None::<String>)
        .await
}
