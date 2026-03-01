use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
    pub args_type: InventionToolArgsType,
    pub call: CallInventionTool,
}

impl InventionTool {
    pub fn new_sync(
        name: &'static str,
        description: &'static str,
        args_type: InventionToolArgsType,
        f: impl Fn(serde_json::Value) -> Result<String, String>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            name,
            description,
            args_type,
            call: Arc::new(move |args| {
                let result = f(args);
                Box::pin(async move { result })
            }),
        }
    }

    pub fn new_async(
        name: &'static str,
        description: &'static str,
        args_type: InventionToolArgsType,
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
            args_type,
            call: Arc::new(move |args| f(args)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventionToolArgsType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    None,
}
