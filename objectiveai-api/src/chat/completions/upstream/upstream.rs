//! Upstream provider enumeration.

/// Returns an iterator over available upstream providers for a request.
pub fn upstreams(
    ensemble_llm: &objectiveai::ensemble_llm::EnsembleLlm,
    request: &super::Params,
) -> Vec<objectiveai::upstream::Upstream> {
    if ensemble_llm.base.model.starts_with("anthropic/")
        && (ensemble_llm
            .base
            .provider
            .as_ref()
            .is_none_or(|p| p.require_parameters.is_none_or(|r| !r)))
    {
        upstreams_filtered(
            &[
                objectiveai::upstream::Upstream::ClaudeAgentSdk,
                objectiveai::upstream::Upstream::OpenRouter,
            ],
            request,
        )
    } else {
        upstreams_filtered(
            &[objectiveai::upstream::Upstream::OpenRouter],
            request,
        )
    }
}

fn upstreams_filtered(
    from_upstreams: &[objectiveai::upstream::Upstream],
    request: &super::Params,
) -> Vec<objectiveai::upstream::Upstream> {
    from_upstreams
        .iter()
        .filter(|upstream| match upstream {
            objectiveai::upstream::Upstream::ClaudeAgentSdk => {
                request.upstreams().map_or(true, |ups| {
                    ups.contains(&objectiveai::upstream::Upstream::ClaudeAgentSdk)
                })
            }
            objectiveai::upstream::Upstream::OpenRouter => {
                request.upstreams().map_or(true, |ups| {
                    ups.contains(&objectiveai::upstream::Upstream::OpenRouter)
                })
            }
            objectiveai::upstream::Upstream::Unknown => false,
        })
        .cloned()
        .collect()
}
