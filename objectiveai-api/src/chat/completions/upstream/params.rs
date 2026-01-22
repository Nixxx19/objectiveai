use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Params {
    Chat {
        request: Arc<objectiveai::chat::completions::request::ChatCompletionCreateParams>,
    },
    Vector {
        request: Arc<objectiveai::vector::completions::request::VectorCompletionCreateParams>,
        vector_pfx_indices: Arc<Vec<(String, usize)>>,
    },
}
