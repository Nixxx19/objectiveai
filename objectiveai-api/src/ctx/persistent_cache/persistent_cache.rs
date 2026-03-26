//! Persistent cache trait for key-value storage.

use objectiveai::error::ResponseError;

/// A persistent cache client for simple key-value storage.
///
/// Implementations may back this with Redis, DynamoDB, filesystem, etc.
#[async_trait::async_trait]
pub trait PersistentCacheClient: Send + Sync + std::fmt::Debug {
    /// Gets a value by namespace and key. Returns `Ok(None)` if the key does not exist.
    async fn get(&self, namespace: &'static str, key: &str) -> Result<Option<String>, ResponseError>;

    /// Sets a value by namespace and key. `permanent` indicates the value will never
    /// change for this key (e.g. content-addressed by commit SHA).
    async fn set(&self, namespace: &'static str, key: &str, value: &str, permanent: bool) -> Result<(), ResponseError>;
}
