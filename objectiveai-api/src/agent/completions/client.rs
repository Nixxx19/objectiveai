use std::sync::Arc;

pub fn response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("agtcpl-{}-{created}", uuid.simple())
}

pub struct Client<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    pub openrouter: OPENROUTER,
    pub claude_agent_sdk: CLAUDEAGENTSDK,
    pub mock: MOCK,
}

impl<OPENROUTER, CLAUDEAGENTSDK, MOCK> Client<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    pub fn new(
        openrouter: OPENROUTER,
        claude_agent_sdk: CLAUDEAGENTSDK,
        mock: MOCK,
    ) -> Self {
        Self {
            openrouter,
            claude_agent_sdk,
            mock,
        }
    }
}

impl<OPENROUTER, CLAUDEAGENTSDK, MOCK> Client<OPENROUTER, CLAUDEAGENTSDK, MOCK>
where
    OPENROUTER: super::UpstreamClient<objectiveai::agent::openrouter::Agent>,
    CLAUDEAGENTSDK: super::UpstreamClient<objectiveai::agent::claude_agent_sdk::Agent>,
    MOCK: super::UpstreamClient<objectiveai::agent::mock::Agent>,
{
    pub async fn create_streaming(
        &self,
        params: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
        continuation: Option<
            super::Continuation<
                OPENROUTER::State,
                CLAUDEAGENTSDK::State,
                MOCK::State,
            >,
        >,
    ) -> Result<
        impl futures::Stream<
            Item = super::StreamItem<
                super::State<
                    OPENROUTER::State,
                    CLAUDEAGENTSDK::State,
                    MOCK::State,
                >,
            >,
        >,
        objectiveai::error::ResponseError,
    > {
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = response_id(created);

        // Placeholder: return an empty stream.
        Ok(futures::stream::empty())
    }
}
