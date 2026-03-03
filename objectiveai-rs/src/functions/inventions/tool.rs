use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use indexmap::IndexMap;

pub type CallInventionTool = Arc<
    dyn Fn(
            serde_json::Value,
        )
            -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct InventionTool {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: IndexMap<String, serde_json::Value>,
    pub call: CallInventionTool,
}

impl InventionTool {
    pub fn new_sync(
        name: &'static str,
        description: &'static str,
        parameters: serde_json::Map<String, serde_json::Value>,
        f: impl Fn(serde_json::Value) -> Result<String, String>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            name,
            description,
            parameters: parameters.into_iter().collect(),
            call: Arc::new(move |args| {
                let result = f(args);
                Box::pin(async move { result })
            }),
        }
    }

    pub fn new_async(
        name: &'static str,
        description: &'static str,
        parameters: serde_json::Map<String, serde_json::Value>,
        f: impl Fn(
            serde_json::Value,
        )
            -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            name,
            description,
            parameters: parameters.into_iter().collect(),
            call: Arc::new(move |args| f(args)),
        }
    }
}
