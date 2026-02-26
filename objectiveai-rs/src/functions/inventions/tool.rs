use std::sync::Arc;

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub args_type: ToolArgsType,
    pub call:
        Arc<dyn Fn(serde_json::Value) -> Result<String, String> + Send + Sync>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolArgsType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    None,
}
