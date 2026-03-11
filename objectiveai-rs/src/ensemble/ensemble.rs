//! Core Ensemble types and validation logic.

use crate::agent;
use crate::vector::completions::request::{Profile, ProfileEntry};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash3_128;
use schemars::JsonSchema;

/// The base configuration for an Ensemble (without computed ID).
///
/// Contains a list of agent configurations that will be validated, deduplicated,
/// and sorted when converting to [`Ensemble`].
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "EnsembleEnsembleBase")]
pub struct EnsembleBase {
    /// The LLMs in this ensemble, with optional counts and fallbacks.
    pub agents: Vec<agent::AgentBaseWithFallbacksAndCount>,
}

/// A validated Ensemble with its computed content-addressed ID.
///
/// Created by converting from [`EnsembleBase`] via [`TryFrom`]. The conversion:
/// 1. Validates and normalizes each agent
/// 2. Merges duplicate LLMs (by full_id) and sums their counts
/// 3. Sorts LLMs by full_id for deterministic ordering
/// 4. Computes the ensemble ID from the sorted (full_id, count) pairs
///
/// # Constraints
///
/// - Individual LLMs with `count: 0` are skipped
/// - Total agent count (sum of all counts) must be between 1 and 128
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "EnsembleEnsemble")]
pub struct Ensemble {
    /// The deterministic content-addressed ID (22-character base62 string).
    pub id: String,
    /// The validated and deduplicated LLMs, sorted by full_id.
    pub agents: Vec<agent::AgentWithFallbacksAndCount>,
}

impl TryFrom<EnsembleBase> for Ensemble {
    type Error = String;
    fn try_from(
        EnsembleBase {
            agents: base_agents,
        }: EnsembleBase,
    ) -> Result<Self, Self::Error> {
        // convert all base LLMs and merge duplicates
        let mut agents_with_full_id: IndexMap<
            String,
            agent::AgentWithFallbacksAndCount,
        > = IndexMap::with_capacity(base_agents.len());
        let mut count = 0;
        for base_agent in base_agents {
            match base_agent.count {
                0 => continue,
                n => count += n,
            }
            let agent: agent::AgentWithFallbacksAndCount =
                base_agent.try_into()?;
            // validate no 2 identical IDs in primary/fallbacks
            if let Some(fallbacks) = &agent.fallbacks {
                if fallbacks.iter().any(|fb| fb.id() == agent.inner.id()) {
                    return Err(format!(
                        "Agent cannot have identical primary and fallback IDs: {}",
                        agent.inner.id()
                    ));
                }
                for i in 0..fallbacks.len() {
                    for j in (i + 1)..fallbacks.len() {
                        if fallbacks[i].id() == fallbacks[j].id() {
                            return Err(format!(
                                "Agent cannot have duplicate fallback IDs: {}",
                                fallbacks[i].id()
                            ));
                        }
                    }
                }
            }
            let full_id = agent.full_id();
            match agents_with_full_id.get_mut(&full_id) {
                Some(existing_agent) => existing_agent.count += agent.count,
                None => {
                    agents_with_full_id.insert(full_id, agent);
                }
            }
        }

        // validate count
        if count == 0 || count > 128 {
            return Err(
                "`ensemble.agents` must contain between 1 and 128 total LLMs"
                    .to_string(),
            );
        }

        // sort by full_id to ensure deterministic order
        agents_with_full_id.sort_unstable_keys();

        // compute ensemble ID
        let mut hasher = XxHash3_128::with_seed(0);
        for (full_id, agent) in &agents_with_full_id {
            hasher.write(full_id.as_bytes());
            let count_bytes = agent.count.to_le_bytes();
            hasher.write(&count_bytes);
        }
        let id = format!("{:0>22}", base62::encode(hasher.finish_128()));

        // collect LLMs
        let agents = agents_with_full_id.into_values().collect::<Vec<_>>();

        // return ensemble
        Ok(Ensemble { id, agents })
    }
}

impl Ensemble {
    /// Converts an EnsembleBase to Ensemble while aligning profile weights.
    ///
    /// Profile weights are filtered (count-0 removed), merged (weighted avg by count),
    /// and sorted to match the resulting Ensemble's agent ordering.
    pub fn try_from_with_profile(
        EnsembleBase {
            agents: base_agents,
        }: EnsembleBase,
        profile: Profile,
    ) -> Result<(Self, Profile), String> {
        // validate lengths match
        if profile.len() != base_agents.len() {
            return Err(format!(
                "profile length ({}) does not match agents length ({})",
                profile.len(),
                base_agents.len()
            ));
        }

        // normalize profile to (weight, invert) pairs
        let profile_pairs = profile.to_weights_and_invert();

        // zip base LLMs with profile entries, filter count-0, validate, and merge
        let mut agents_with_full_id: IndexMap<
            String,
            (
                agent::AgentWithFallbacksAndCount,
                Decimal, // weighted sum (weight * count)
                u64,     // total count (for computing weighted average)
                bool,    // invert
            ),
        > = IndexMap::with_capacity(base_agents.len());
        let mut count = 0u64;

        for (base_agent, (weight, invert)) in
            base_agents.into_iter().zip(profile_pairs.into_iter())
        {
            match base_agent.count {
                0 => continue,
                n => count += n,
            }
            let agent: agent::AgentWithFallbacksAndCount =
                base_agent.try_into()?;
            // validate no 2 identical IDs in primary/fallbacks
            if let Some(fallbacks) = &agent.fallbacks {
                if fallbacks.iter().any(|fb| fb.id() == agent.inner.id()) {
                    return Err(format!(
                        "Agent cannot have identical primary and fallback IDs: {}",
                        agent.inner.id()
                    ));
                }
                for i in 0..fallbacks.len() {
                    for j in (i + 1)..fallbacks.len() {
                        if fallbacks[i].id() == fallbacks[j].id() {
                            return Err(format!(
                                "Agent cannot have duplicate fallback IDs: {}",
                                fallbacks[i].id()
                            ));
                        }
                    }
                }
            }
            let full_id = agent.full_id();
            match agents_with_full_id.get_mut(&full_id) {
                Some((
                    existing,
                    weighted_sum,
                    total_count,
                    existing_invert,
                )) => {
                    if *existing_invert != invert {
                        return Err(format!(
                            "conflicting invert flags for merged agent with full_id: {}",
                            full_id
                        ));
                    }
                    *weighted_sum += weight * Decimal::from(agent.count);
                    *total_count += agent.count;
                    existing.count += agent.count;
                }
                None => {
                    let weighted_sum = weight * Decimal::from(agent.count);
                    let total_count = agent.count;
                    agents_with_full_id.insert(
                        full_id,
                        (agent, weighted_sum, total_count, invert),
                    );
                }
            }
        }

        // validate count
        if count == 0 || count > 128 {
            return Err(
                "`ensemble.agents` must contain between 1 and 128 total LLMs"
                    .to_string(),
            );
        }

        // sort by full_id to ensure deterministic order
        agents_with_full_id.sort_unstable_keys();

        // compute ensemble ID
        let mut hasher = XxHash3_128::with_seed(0);
        for (full_id, (agent, _, _, _)) in &agents_with_full_id {
            hasher.write(full_id.as_bytes());
            let count_bytes = agent.count.to_le_bytes();
            hasher.write(&count_bytes);
        }
        let id = format!("{:0>22}", base62::encode(hasher.finish_128()));

        // collect LLMs and aligned profile entries
        let mut agents = Vec::with_capacity(agents_with_full_id.len());
        let mut entries = Vec::with_capacity(agents_with_full_id.len());
        for (_, (agent, weighted_sum, total_count, invert)) in
            agents_with_full_id
        {
            agents.push(agent);
            let merged_weight = weighted_sum / Decimal::from(total_count);
            entries.push(ProfileEntry {
                weight: merged_weight,
                invert: if invert { Some(true) } else { None },
            });
        }

        Ok((Ensemble { id, agents }, Profile::Entries(entries)))
    }
}

