use serde::{Deserialize, Serialize};

pub trait StatusError {
    fn status(&self) -> u16;
    fn message(&self) -> Option<serde_json::Value>;
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("{}", &serde_json::to_string(self).unwrap_or_default())]
pub struct ResponseError {
    pub code: u16,
    pub message: serde_json::Value,
}

impl StatusError for ResponseError {
    fn status(&self) -> u16 {
        self.code
    }

    fn message(&self) -> Option<serde_json::Value> {
        Some(self.message.clone())
    }
}

impl<T> From<&T> for ResponseError
where
    T: StatusError,
{
    fn from(error: &T) -> Self {
        ResponseError {
            code: error.status(),
            message: error.message().unwrap_or_default(),
        }
    }
}
