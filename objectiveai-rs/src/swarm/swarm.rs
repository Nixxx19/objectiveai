//! Core Swarm types and validation logic.

use crate::agent;
use crate::vector::completions::request::{Profile, ProfileEntry};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twox_hash::XxHash3_128;
use schemars::JsonSchema;

// ── Pre-validation types (no computed ID) ──────────────────────────

/// An inline swarm base definition (without computed ID or metadata).
///
/// Contains a list of agent configurations that will be validated, deduplicated,
/// and sorted when converting to an [`InlineSwarm`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "swarm.InlineSwarmBase")]
pub struct InlineSwarmBase {
    /// The LLMs in this swarm, with optional counts and fallbacks.
    pub agents: Vec<agent::InlineAgentBaseWithFallbacksOrRemoteWithCount>,
}

impl InlineSwarmBase {
    /// Validates and converts to an [`InlineSwarm`] with computed ID.
    ///
    /// Remote agent references are resolved from the provided hashmap.
    /// The hashmap key format is `"{owner}/{repository}/{commit}"`.
    pub fn convert(
        self,
        remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
    ) -> Result<InlineSwarm, String> {
        convert_base(self.agents, remote_agents)
    }
}

/// A remote swarm base definition with metadata (without computed ID).
///
/// Like [`InlineSwarmBase`] but includes a description for remote storage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.RemoteSwarmBase")]
pub struct RemoteSwarmBase {
    /// Human-readable description of what this swarm does.
    pub description: String,
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<InlineSwarmBase>")]
    pub inner: InlineSwarmBase,
}

impl RemoteSwarmBase {
    /// Validates and converts to a [`RemoteSwarm`] with computed ID.
    pub fn convert(
        self,
        remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
    ) -> Result<RemoteSwarm, String> {
        Ok(RemoteSwarm {
            description: self.description,
            inner: self.inner.convert(remote_agents)?,
        })
    }
}

/// A swarm base definition, either remote (with metadata) or inline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "swarm.SwarmBase")]
pub enum SwarmBase {
    Remote(RemoteSwarmBase),
    Inline(InlineSwarmBase),
}

impl SwarmBase {
    /// Validates and converts to a [`Swarm`] with computed ID.
    pub fn convert(
        self,
        remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
    ) -> Result<Swarm, String> {
        match self {
            SwarmBase::Remote(r) => Ok(Swarm::Remote(r.convert(remote_agents)?)),
            SwarmBase::Inline(i) => Ok(Swarm::Inline(i.convert(remote_agents)?)),
        }
    }
}

// ── Post-validation types (with computed ID) ───────────────────────

/// A validated inline Swarm with its computed content-addressed ID.
///
/// Created by converting from [`InlineSwarmBase`] via [`InlineSwarmBase::convert`].
/// The conversion:
/// 1. Validates and normalizes each agent
/// 2. Merges duplicate LLMs (by full_id) and sums their counts
/// 3. Sorts LLMs by full_id for deterministic ordering
/// 4. Computes the swarm ID from the sorted (full_id, count) pairs
///
/// # Constraints
///
/// - Individual LLMs with `count: 0` are skipped
/// - Total agent count (sum of all counts) must be between 1 and 128
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.InlineSwarm")]
pub struct InlineSwarm {
    /// The deterministic content-addressed ID (22-character base62 string).
    pub id: String,
    /// The validated and deduplicated LLMs, sorted by full_id.
    pub agents: Vec<agent::AgentWithFallbacksWithCount>,
}

/// A validated remote Swarm with metadata and computed content-addressed ID.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.RemoteSwarm")]
pub struct RemoteSwarm {
    pub description: String,
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<InlineSwarm>")]
    pub inner: InlineSwarm,
}

/// A validated Swarm, either remote (with metadata) or inline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "swarm.Swarm")]
pub enum Swarm {
    Remote(RemoteSwarm),
    Inline(InlineSwarm),
}

// ── WithProfile types ──────────────────────────────────────────────

/// An [`InlineSwarmBase`] with a profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.InlineSwarmBaseWithProfile")]
pub struct InlineSwarmBaseWithProfile {
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<InlineSwarmBase>")]
    pub inner: InlineSwarmBase,
    pub profile: Profile,
}

impl InlineSwarmBaseWithProfile {
    /// Converts to an [`InlineSwarmWithProfile`] with computed ID and aligned profile.
    pub fn convert(
        self,
        remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
    ) -> Result<InlineSwarmWithProfile, String> {
        let (inner, profile) = convert_with_profile(
            self.inner.agents,
            self.profile,
            remote_agents,
        )?;
        Ok(InlineSwarmWithProfile { inner, profile })
    }
}

/// A [`RemoteSwarmBase`] with a profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.RemoteSwarmBaseWithProfile")]
pub struct RemoteSwarmBaseWithProfile {
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<RemoteSwarmBase>")]
    pub inner: RemoteSwarmBase,
    pub profile: Profile,
}

impl RemoteSwarmBaseWithProfile {
    /// Converts to a [`RemoteSwarmWithProfile`] with computed ID and aligned profile.
    pub fn convert(
        self,
        remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
    ) -> Result<RemoteSwarmWithProfile, String> {
        let (inner, profile) = convert_with_profile(
            self.inner.inner.agents,
            self.profile,
            remote_agents,
        )?;
        Ok(RemoteSwarmWithProfile {
            inner: RemoteSwarm {
                description: self.inner.description,
                inner,
            },
            profile,
        })
    }
}

/// A [`SwarmBase`] with a profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.SwarmBaseWithProfile")]
pub struct SwarmBaseWithProfile {
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<SwarmBase>")]
    pub inner: SwarmBase,
    pub profile: Profile,
}

impl SwarmBaseWithProfile {
    /// Converts to a [`SwarmWithProfile`] with computed ID and aligned profile.
    pub fn convert(
        self,
        remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
    ) -> Result<SwarmWithProfile, String> {
        match self.inner {
            SwarmBase::Remote(r) => {
                let wp = RemoteSwarmBaseWithProfile { inner: r, profile: self.profile };
                let converted = wp.convert(remote_agents)?;
                Ok(SwarmWithProfile {
                    inner: Swarm::Remote(converted.inner),
                    profile: converted.profile,
                })
            }
            SwarmBase::Inline(i) => {
                let wp = InlineSwarmBaseWithProfile { inner: i, profile: self.profile };
                let converted = wp.convert(remote_agents)?;
                Ok(SwarmWithProfile {
                    inner: Swarm::Inline(converted.inner),
                    profile: converted.profile,
                })
            }
        }
    }
}

/// A validated [`InlineSwarm`] with an aligned profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.InlineSwarmWithProfile")]
pub struct InlineSwarmWithProfile {
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<InlineSwarm>")]
    pub inner: InlineSwarm,
    pub profile: Profile,
}

/// A validated [`RemoteSwarm`] with an aligned profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.RemoteSwarmWithProfile")]
pub struct RemoteSwarmWithProfile {
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<RemoteSwarm>")]
    pub inner: RemoteSwarm,
    pub profile: Profile,
}

/// A validated [`Swarm`] with an aligned profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.SwarmWithProfile")]
pub struct SwarmWithProfile {
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<Swarm>")]
    pub inner: Swarm,
    pub profile: Profile,
}

// ── InlineSwarmBaseWithProfileOrRemote ──────────────────────────────

/// A swarm specification that is either an inline swarm base with profile
/// or a remote path reference.
///
/// Used to allow swarms to be specified inline (with optional profile)
/// or resolved from a remote source via a hashmap during conversion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "swarm.InlineSwarmBaseWithProfileOrRemote")]
pub enum InlineSwarmBaseWithProfileOrRemote {
    SwarmBase(InlineSwarmBaseWithProfile),
    Remote(crate::RemotePath),
}

// ── Private helpers ────────────────────────────────────────────────

/// Validates agent fallbacks for duplicate IDs.
fn validate_agent_fallbacks(agent: &agent::AgentWithFallbacks) -> Result<(), String> {
    let inline = match agent {
        agent::AgentWithFallbacks::Remote(a) => &a.inner,
        agent::AgentWithFallbacks::Inline(a) => a,
    };
    if let Some(fallbacks) = &inline.fallbacks {
        if fallbacks.iter().any(|fb| fb.id() == inline.inner.id()) {
            return Err(format!(
                "Agent cannot have identical primary and fallback IDs: {}",
                inline.inner.id()
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
    Ok(())
}

/// Converts an agent slot (inline or remote reference) to a validated agent.
fn convert_agent_slot(
    slot: agent::InlineAgentBaseWithFallbacksOrRemote,
    remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
) -> Result<agent::AgentWithFallbacks, String> {
    match slot {
        agent::InlineAgentBaseWithFallbacksOrRemote::AgentBase(base_with_fallbacks) => {
            Ok(agent::AgentWithFallbacks::Inline(base_with_fallbacks.convert()?))
        }
        agent::InlineAgentBaseWithFallbacksOrRemote::Remote(path) => {
            let remote_agents = remote_agents.ok_or_else(|| {
                format!(
                    "remote agent reference '{}/{}/{}' but no agents hashmap provided",
                    path.owner, path.repository, path.commit
                )
            })?;
            let key = format!("{}/{}/{}", path.owner, path.repository, path.commit);
            let agent = remote_agents.get(&key).ok_or_else(|| {
                format!(
                    "remote agent '{}/{}/{}' not found in agents hashmap",
                    path.owner, path.repository, path.commit
                )
            })?;
            Ok(agent::AgentWithFallbacks::Remote(agent.clone()))
        }
    }
}

/// Core conversion: validates agents, deduplicates, sorts, computes ID.
fn convert_base(
    agents: Vec<agent::InlineAgentBaseWithFallbacksOrRemoteWithCount>,
    remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
) -> Result<InlineSwarm, String> {
    let mut agents_with_full_id: IndexMap<
        String,
        agent::AgentWithFallbacksWithCount,
    > = IndexMap::with_capacity(agents.len());
    let mut count = 0;
    for base_agent in agents {
        match base_agent.count {
            0 => continue,
            n => count += n,
        }
        let converted = convert_agent_slot(base_agent.inner, remote_agents)?;
        validate_agent_fallbacks(&converted)?;
        let agent_with_count = agent::AgentWithFallbacksWithCount {
            count: base_agent.count,
            inner: converted,
        };
        merge_agent(&mut agents_with_full_id, agent_with_count);
    }

    finalize_swarm(count, agents_with_full_id)
}

/// Core conversion with profile alignment.
fn convert_with_profile(
    agents: Vec<agent::InlineAgentBaseWithFallbacksOrRemoteWithCount>,
    profile: Profile,
    remote_agents: Option<&HashMap<String, agent::RemoteAgentWithFallbacks>>,
) -> Result<(InlineSwarm, Profile), String> {
    if profile.len() != agents.len() {
        return Err(format!(
            "profile length ({}) does not match agents length ({})",
            profile.len(),
            agents.len()
        ));
    }

    let profile_pairs = profile.to_weights_and_invert();

    let mut agents_with_full_id: IndexMap<
        String,
        (
            agent::AgentWithFallbacksWithCount,
            Decimal,
            u64,
            bool,
        ),
    > = IndexMap::with_capacity(agents.len());
    let mut count = 0u64;

    for (base_agent, (weight, invert)) in
        agents.into_iter().zip(profile_pairs.into_iter())
    {
        match base_agent.count {
            0 => continue,
            n => count += n,
        }
        let converted = convert_agent_slot(base_agent.inner, remote_agents)?;
        validate_agent_fallbacks(&converted)?;
        let full_id = converted.full_id();
        let agent_with_count = agent::AgentWithFallbacksWithCount {
            count: base_agent.count,
            inner: converted,
        };
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
                *weighted_sum += weight * Decimal::from(agent_with_count.count);
                *total_count += agent_with_count.count;
                existing.count += agent_with_count.count;
            }
            None => {
                let weighted_sum = weight * Decimal::from(agent_with_count.count);
                let total_count = agent_with_count.count;
                agents_with_full_id.insert(
                    full_id,
                    (agent_with_count, weighted_sum, total_count, invert),
                );
            }
        }
    }

    if count == 0 || count > 128 {
        return Err(
            "`swarm.agents` must contain between 1 and 128 total LLMs"
                .to_string(),
        );
    }

    agents_with_full_id.sort_unstable_keys();

    let mut hasher = XxHash3_128::with_seed(0);
    for (full_id, (agent, _, _, _)) in &agents_with_full_id {
        hasher.write(full_id.as_bytes());
        let count_bytes = agent.count.to_le_bytes();
        hasher.write(&count_bytes);
    }
    let id = format!("{:0>22}", base62::encode(hasher.finish_128()));

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

    Ok((InlineSwarm { id, agents }, Profile::Entries(entries)))
}

/// Finalize: validate count, sort, compute ID, collect agents.
fn finalize_swarm(
    count: u64,
    agents_with_full_id: IndexMap<String, agent::AgentWithFallbacksWithCount>,
) -> Result<InlineSwarm, String> {
    if count == 0 || count > 128 {
        return Err(
            "`swarm.agents` must contain between 1 and 128 total LLMs"
                .to_string(),
        );
    }

    let mut agents_with_full_id = agents_with_full_id;
    agents_with_full_id.sort_unstable_keys();

    let mut hasher = XxHash3_128::with_seed(0);
    for (full_id, agent) in &agents_with_full_id {
        hasher.write(full_id.as_bytes());
        let count_bytes = agent.count.to_le_bytes();
        hasher.write(&count_bytes);
    }
    let id = format!("{:0>22}", base62::encode(hasher.finish_128()));

    let agents = agents_with_full_id.into_values().collect::<Vec<_>>();

    Ok(InlineSwarm { id, agents })
}

/// Merge a validated agent into the dedup map.
fn merge_agent(
    agents_with_full_id: &mut IndexMap<String, agent::AgentWithFallbacksWithCount>,
    agent_with_count: agent::AgentWithFallbacksWithCount,
) {
    let full_id = agent_with_count.inner.full_id();
    match agents_with_full_id.get_mut(&full_id) {
        Some(existing) => existing.count += agent_with_count.count,
        None => {
            agents_with_full_id.insert(full_id, agent_with_count);
        }
    }
}
