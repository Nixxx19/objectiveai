#[cfg(not(target_arch = "wasm32"))]
pub trait ResponseErrorExt {
    fn into_response(self) -> axum::response::Response;
}

#[cfg(not(target_arch = "wasm32"))]
impl ResponseErrorExt for objectiveai::error::ResponseError {
    fn into_response(self) -> axum::response::Response {
        use axum::response::IntoResponse;
        let status = axum::http::StatusCode::from_u16(self.code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_string(&self).unwrap_or_default();
        println!("ResponseError: {}", body);
        (status, body).into_response()
    }
}
