/// Returns available upstream providers for a given ensemble LLM.
///
/// - Anthropic models (without `require_parameters`) get `ClaudeAgentSdk` + `OpenRouter`.
/// - Everything else gets `OpenRouter` only.
/// - If `requested` is `Some`, results are filtered to only include requested upstreams.
pub fn route(
    ensemble_llm: &crate::ensemble_llm::EnsembleLlm,
    requested: Option<&[super::Upstream]>,
) -> Vec<super::Upstream> {
    let available = if ensemble_llm.base.model.starts_with("anthropic/")
        && ensemble_llm
            .base
            .provider
            .as_ref()
            .is_none_or(|p| p.require_parameters.is_none_or(|r| !r))
    {
        &[
            super::Upstream::ClaudeAgentSdk,
            super::Upstream::OpenRouter,
        ][..]
    } else {
        &[super::Upstream::OpenRouter][..]
    };

    available
        .iter()
        .filter(|upstream| {
            if let super::Upstream::Unknown = upstream {
                return false;
            }
            requested.map_or(true, |req| req.contains(upstream))
        })
        .copied()
        .collect()
}
