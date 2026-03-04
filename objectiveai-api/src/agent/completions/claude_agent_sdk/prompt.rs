use super::super::ContinuationItem;

pub struct Prompt {
    pub system_prompt: Option<String>,
    pub message: super::sdk_message::SDKUserMessage,
}

impl Prompt {
    pub fn new(
        messages: &[objectiveai::agent::completions::message::Message],
        continuation: Option<&[ContinuationItem<super::State>]>,
    ) -> Result<Self, super::Error> {
        unimplemented!()
    }
}
