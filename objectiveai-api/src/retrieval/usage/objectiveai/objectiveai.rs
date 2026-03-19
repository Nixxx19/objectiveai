//! ObjectiveAI usage statistics implementation.

use crate::ctx;
use objectiveai::error::ResponseError;
use std::sync::Arc;

pub struct ObjectiveAiClient {
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiClient {
    pub fn new(client: Arc<crate::objectiveai_http::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for ObjectiveAiClient
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn get_agent_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::agent::request::GetAgentRequest,
    ) -> Result<objectiveai::agent::response::UsageAgentResponse, ResponseError> {
        let client = self.objectiveai_client(ctx).await;
        objectiveai::agent::get_agent_usage(&client, params.clone())
            .await
            .map_err(|e| ResponseError::from(&e))
    }

    async fn get_swarm_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::swarm::request::GetSwarmRequest,
    ) -> Result<objectiveai::swarm::response::UsageSwarmResponse, ResponseError> {
        let client = self.objectiveai_client(ctx).await;
        objectiveai::swarm::get_swarm_usage(&client, params.clone())
            .await
            .map_err(|e| ResponseError::from(&e))
    }

    async fn get_function_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::request::GetFunctionRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionResponse, ResponseError> {
        let client = self.objectiveai_client(ctx).await;
        objectiveai::functions::get_function_usage(&client, params.clone())
            .await
            .map_err(|e| ResponseError::from(&e))
    }

    async fn get_profile_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::profiles::request::GetProfileRequest,
    ) -> Result<objectiveai::functions::profiles::response::UsageProfileResponse, ResponseError>
    {
        let client = self.objectiveai_client(ctx).await;
        objectiveai::functions::profiles::get_profile_usage(&client, params.clone())
            .await
            .map_err(|e| ResponseError::from(&e))
    }

    async fn get_function_profile_pair_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::request::GetFunctionProfilePairUsageRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionProfilePairResponse, ResponseError>
    {
        let client = self.objectiveai_client(ctx).await;
        objectiveai::functions::get_function_profile_pair_usage(&client, params.clone())
            .await
            .map_err(|e| ResponseError::from(&e))
    }
}

impl ObjectiveAiClient {
    async fn objectiveai_client<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> objectiveai::HttpClient {
        self.client.with_authorization(ctx).await
    }
}
