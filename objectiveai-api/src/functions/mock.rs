//! Shared mock data for Agents, Swarms, Functions, and Profiles.
//!
//! Provides embedded JSON fixtures used by the retrieval module.

/// Returns a mock Agent by name.
pub fn get_agent(
    name: &str,
) -> Option<objectiveai::agent::RemoteAgentBaseWithFallbacks> {
    let json = get_agent_json(name)?;
    Some(serde_json::from_str(json).expect("invalid mock agent JSON"))
}

/// Returns mock Agent JSON by name.
fn get_agent_json(name: &str) -> Option<&'static str> {
    match name {
        "mock-agent-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/agents/mock-agent-1.json"))),
        "mock-agent-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/agents/mock-agent-2.json"))),
        "mock-agent-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/agents/mock-agent-3.json"))),
        "mock-agent-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/agents/mock-agent-4.json"))),
        _ => None,
    }
}

/// All mock Agent names.
const AGENT_REPOSITORIES: &[&str] = &["mock-agent-1", "mock-agent-2", "mock-agent-3", "mock-agent-4"];

/// Lists all mock Agents.
pub fn list_agents() -> objectiveai::agent::response::ListAgentResponse {
    objectiveai::agent::response::ListAgentResponse {
        data: AGENT_REPOSITORIES
            .iter()
            .map(|name| objectiveai::RemotePath::Mock {
                name: name.to_string(),
            })
            .collect(),
    }
}

/// Returns a mock Swarm by name.
pub fn get_swarm(
    name: &str,
) -> Option<objectiveai::swarm::RemoteSwarmBase> {
    let json = get_swarm_json(name)?;
    Some(serde_json::from_str(json).expect("invalid mock swarm JSON"))
}

/// Returns mock Swarm JSON by name.
fn get_swarm_json(name: &str) -> Option<&'static str> {
    match name {
        "mock-swarm-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/swarms/mock-swarm-1.json"))),
        "mock-swarm-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/swarms/mock-swarm-2.json"))),
        _ => None,
    }
}

/// All mock Swarm names.
const SWARM_REPOSITORIES: &[&str] = &["mock-swarm-1", "mock-swarm-2"];

/// Lists all mock Swarms.
pub fn list_swarms() -> objectiveai::swarm::response::ListSwarmResponse {
    objectiveai::swarm::response::ListSwarmResponse {
        data: SWARM_REPOSITORIES
            .iter()
            .map(|name| objectiveai::RemotePath::Mock {
                name: name.to_string(),
            })
            .collect(),
    }
}

/// Returns a mock Function by name.
pub fn get_function(
    name: &str,
) -> Option<objectiveai::functions::FullRemoteFunction> {
    let json = get_function_json(name)?;
    Some(serde_json::from_str(json).expect("invalid mock function JSON"))
}

/// Returns mock Function JSON by repository name.
fn get_function_json(repository: &str) -> Option<&'static str> {
    match repository {
        "mock-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-1.json"))),
        "mock-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-2.json"))),
        "mock-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-3.json"))),
        "mock-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-4.json"))),
        "mock-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-5.json"))),
        "mock-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-6.json"))),
        "mock-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-7.json"))),
        "mock-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-8.json"))),
        "mock-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-9.json"))),
        "mock-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-10.json"))),
        "mock-11" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-11.json"))),
        "mock-12" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-12.json"))),
        "mock-13" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-13.json"))),
        "mock-14" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-14.json"))),
        "mock-15" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-15.json"))),
        "mock-16" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-16.json"))),
        "mock-17" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-17.json"))),
        "mock-18" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-18.json"))),
        "mock-19" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-19.json"))),
        "mock-20" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-20.json"))),
        "mock-21" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-21.json"))),
        "mock-22" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-22.json"))),
        "mock-23" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-23.json"))),
        "mock-24" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-24.json"))),
        "mock-25" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-25.json"))),
        "mock-err-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-1.json"))),
        "mock-err-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-2.json"))),
        "mock-err-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-3.json"))),
        "mock-err-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-4.json"))),
        "mock-err-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-5.json"))),
        "mock-err-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-6.json"))),
        "mock-err-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-7.json"))),
        "mock-err-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-8.json"))),
        "mock-err-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-9.json"))),
        "mock-err-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/functions/mock-err-10.json"))),
        _ => None,
    }
}

/// All mock Function repository names.
const FUNCTION_REPOSITORIES: &[&str] = &[
    "mock-1", "mock-2", "mock-3", "mock-4", "mock-5",
    "mock-6", "mock-7", "mock-8", "mock-9", "mock-10",
    "mock-11", "mock-12", "mock-13", "mock-14", "mock-15",
    "mock-16", "mock-17", "mock-18", "mock-19", "mock-20",
    "mock-21", "mock-22", "mock-23", "mock-24", "mock-25",
    "mock-err-1", "mock-err-2", "mock-err-3", "mock-err-4", "mock-err-5",
    "mock-err-6", "mock-err-7", "mock-err-8", "mock-err-9", "mock-err-10",
];

/// Lists all mock Functions.
pub fn list_functions() -> objectiveai::functions::response::ListFunctionResponse {
    objectiveai::functions::response::ListFunctionResponse {
        data: FUNCTION_REPOSITORIES
            .iter()
            .map(|repo| objectiveai::RemotePath::Mock {
                name: repo.to_string(),
            })
            .collect(),
    }
}

/// Returns a mock Profile by name.
pub fn get_profile(
    name: &str,
) -> Option<objectiveai::functions::RemoteProfile> {
    let json = get_profile_json(name)?;
    Some(serde_json::from_str(json).expect("invalid mock profile JSON"))
}

/// Returns mock Profile JSON by repository name.
fn get_profile_json(repository: &str) -> Option<&'static str> {
    match repository {
        "mock-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-1.json"))),
        "mock-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-2.json"))),
        "mock-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-3.json"))),
        "mock-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-4.json"))),
        "mock-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-5.json"))),
        "mock-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-6.json"))),
        "mock-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-7.json"))),
        "mock-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-8.json"))),
        "mock-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-9.json"))),
        "mock-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-10.json"))),
        "mock-11" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-11.json"))),
        "mock-12" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-12.json"))),
        "mock-13" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-13.json"))),
        "mock-14" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-14.json"))),
        "mock-15" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-15.json"))),
        "mock-16" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-16.json"))),
        "mock-17" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-17.json"))),
        "mock-18" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-18.json"))),
        "mock-19" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-19.json"))),
        "mock-20" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-20.json"))),
        "mock-21" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-21.json"))),
        "mock-22" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-22.json"))),
        "mock-23" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-23.json"))),
        "mock-24" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-24.json"))),
        "mock-err-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-1.json"))),
        "mock-err-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-2.json"))),
        "mock-err-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-3.json"))),
        "mock-err-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-4.json"))),
        "mock-err-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-5.json"))),
        "mock-err-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-6.json"))),
        "mock-err-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-7.json"))),
        "mock-err-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-8.json"))),
        "mock-err-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-9.json"))),
        "mock-err-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-10.json"))),
        "mock-err-11" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-11.json"))),
        "mock-err-12" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-12.json"))),
        "mock-err-13" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-13.json"))),
        "mock-err-14" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-14.json"))),
        "mock-err-15" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-15.json"))),
        "mock-err-16" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-16.json"))),
        "mock-err-17" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-17.json"))),
        "mock-err-18" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mock/profiles/mock-err-18.json"))),
        _ => None,
    }
}

/// All mock Profile repository names.
const PROFILE_REPOSITORIES: &[&str] = &[
    "mock-1", "mock-2", "mock-3", "mock-4", "mock-5",
    "mock-6", "mock-7", "mock-8", "mock-9", "mock-10",
    "mock-11", "mock-12", "mock-13", "mock-14", "mock-15",
    "mock-16", "mock-17", "mock-18", "mock-19", "mock-20",
    "mock-21", "mock-22", "mock-23", "mock-24",
    "mock-err-1", "mock-err-2", "mock-err-3", "mock-err-4", "mock-err-5",
    "mock-err-6", "mock-err-7", "mock-err-8", "mock-err-9", "mock-err-10",
    "mock-err-11", "mock-err-12", "mock-err-13", "mock-err-14", "mock-err-15",
    "mock-err-16", "mock-err-17", "mock-err-18",
];

/// Lists all mock Profiles.
pub fn list_profiles() -> objectiveai::functions::profiles::response::ListProfileResponse {
    objectiveai::functions::profiles::response::ListProfileResponse {
        data: PROFILE_REPOSITORIES
            .iter()
            .map(|repo| objectiveai::RemotePath::Mock {
                name: repo.to_string(),
            })
            .collect(),
    }
}
