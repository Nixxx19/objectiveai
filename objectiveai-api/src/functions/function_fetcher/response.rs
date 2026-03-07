//! Response types for function fetchers.

use serde::{Deserialize, Serialize};

/// A fetched function that may contain alpha function types.
///
/// This is the internal fetcher response type. Alpha functions are transpiled
/// to standard functions before being returned from the router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullGetFunction {
    pub remote: objectiveai::functions::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
    #[serde(flatten)]
    pub inner: objectiveai::functions::FullRemoteFunction,
}

impl FullGetFunction {
    /// Transpiles alpha function types to standard function types.
    pub fn transpile(self) -> objectiveai::functions::response::GetFunction {
        objectiveai::functions::response::GetFunction {
            remote: self.remote,
            owner: self.owner,
            repository: self.repository,
            commit: self.commit,
            inner: self.inner.transpile(),
        }
    }
}
