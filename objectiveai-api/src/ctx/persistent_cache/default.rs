//! Default (no-op) persistent cache client.

use objectiveai::error::ResponseError;

/// A no-op persistent cache client that never stores or retrieves anything.
#[derive(Debug)]
pub struct DefaultPersistentCacheClient;

#[async_trait::async_trait]
impl super::PersistentCacheClient for DefaultPersistentCacheClient {
    async fn get(&self, _key: &str) -> Result<Option<String>, ResponseError> {
        Ok(None)
    }

    async fn set(&self, _key: &str, _value: &str, _permanent: bool) -> Result<(), ResponseError> {
        Ok(())
    }
}
