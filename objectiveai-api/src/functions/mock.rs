//! Shared mock data for Functions and Profiles.
//!
//! Provides embedded JSON fixtures used by both the fetchers and list clients.

/// Returns a mock Function by owner/repository/commit.
///
/// Returns `None` if owner or commit is not `"mock"`, or if the repository
/// name is unrecognized.
pub fn get_function(
    owner: &str,
    repository: &str,
    commit: Option<&str>,
) -> Option<objectiveai::functions::FullRemoteFunction> {
    if owner != "mock" || commit != Some("mock") {
        return None;
    }
    let json = get_function_json(repository)?;
    Some(serde_json::from_str(json).expect("invalid mock function JSON"))
}

/// Returns mock Function JSON by repository name.
fn get_function_json(repository: &str) -> Option<&'static str> {
    match repository {
        "mock-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-1.json"))),
        "mock-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-2.json"))),
        "mock-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-3.json"))),
        "mock-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-4.json"))),
        "mock-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-5.json"))),
        "mock-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-6.json"))),
        "mock-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-7.json"))),
        "mock-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-8.json"))),
        "mock-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-9.json"))),
        "mock-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-10.json"))),
        "mock-11" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-11.json"))),
        "mock-12" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-12.json"))),
        "mock-13" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-13.json"))),
        "mock-14" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-14.json"))),
        "mock-15" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-15.json"))),
        "mock-16" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-16.json"))),
        "mock-17" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-17.json"))),
        "mock-18" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-18.json"))),
        "mock-19" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-19.json"))),
        "mock-20" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-20.json"))),
        "mock-21" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-21.json"))),
        "mock-err-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-1.json"))),
        "mock-err-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-2.json"))),
        "mock-err-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-3.json"))),
        "mock-err-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-4.json"))),
        "mock-err-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-5.json"))),
        "mock-err-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-6.json"))),
        "mock-err-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-7.json"))),
        "mock-err-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-8.json"))),
        "mock-err-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-9.json"))),
        "mock-err-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/function_fetcher/mock/mock-err-10.json"))),
        _ => None,
    }
}

/// All mock Function repository names.
const FUNCTION_REPOSITORIES: &[&str] = &[
    "mock-1", "mock-2", "mock-3", "mock-4", "mock-5",
    "mock-6", "mock-7", "mock-8", "mock-9", "mock-10",
    "mock-11", "mock-12", "mock-13", "mock-14", "mock-15",
    "mock-16", "mock-17", "mock-18", "mock-19", "mock-20",
    "mock-21",
    "mock-err-1", "mock-err-2", "mock-err-3", "mock-err-4", "mock-err-5",
    "mock-err-6", "mock-err-7", "mock-err-8", "mock-err-9", "mock-err-10",
];

/// Lists all mock Functions.
pub fn list_functions() -> objectiveai::functions::response::ListFunction {
    objectiveai::functions::response::ListFunction {
        data: FUNCTION_REPOSITORIES
            .iter()
            .map(|repo| objectiveai::functions::response::ListFunctionItem {
                remote: objectiveai::functions::Remote::Mock,
                owner: "mock".to_string(),
                repository: repo.to_string(),
                commit: "mock".to_string(),
            })
            .collect(),
    }
}

/// Returns a mock Profile by owner/repository/commit.
///
/// Returns `None` if owner or commit is not `"mock"`, or if the repository
/// name is unrecognized.
pub fn get_profile(
    owner: &str,
    repository: &str,
    commit: Option<&str>,
) -> Option<objectiveai::functions::RemoteProfile> {
    if owner != "mock" || commit != Some("mock") {
        return None;
    }
    let json = get_profile_json(repository)?;
    Some(serde_json::from_str(json).expect("invalid mock profile JSON"))
}

/// Returns mock Profile JSON by repository name.
fn get_profile_json(repository: &str) -> Option<&'static str> {
    match repository {
        "mock-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-1.json"))),
        "mock-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-2.json"))),
        "mock-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-3.json"))),
        "mock-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-4.json"))),
        "mock-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-5.json"))),
        "mock-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-6.json"))),
        "mock-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-7.json"))),
        "mock-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-8.json"))),
        "mock-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-9.json"))),
        "mock-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-10.json"))),
        "mock-11" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-11.json"))),
        "mock-12" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-12.json"))),
        "mock-13" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-13.json"))),
        "mock-14" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-14.json"))),
        "mock-15" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-15.json"))),
        "mock-16" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-16.json"))),
        "mock-17" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-17.json"))),
        "mock-18" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-18.json"))),
        "mock-19" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-19.json"))),
        "mock-20" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-20.json"))),
        "mock-21" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-21.json"))),
        "mock-err-1" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-1.json"))),
        "mock-err-2" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-2.json"))),
        "mock-err-3" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-3.json"))),
        "mock-err-4" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-4.json"))),
        "mock-err-5" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-5.json"))),
        "mock-err-6" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-6.json"))),
        "mock-err-7" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-7.json"))),
        "mock-err-8" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-8.json"))),
        "mock-err-9" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-9.json"))),
        "mock-err-10" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-10.json"))),
        "mock-err-11" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-11.json"))),
        "mock-err-12" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-12.json"))),
        "mock-err-13" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-13.json"))),
        "mock-err-14" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-14.json"))),
        "mock-err-15" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-15.json"))),
        "mock-err-16" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-16.json"))),
        "mock-err-17" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-17.json"))),
        "mock-err-18" => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/profile_fetcher/mock/mock-err-18.json"))),
        _ => None,
    }
}

/// All mock Profile repository names.
const PROFILE_REPOSITORIES: &[&str] = &[
    "mock-1", "mock-2", "mock-3", "mock-4", "mock-5",
    "mock-6", "mock-7", "mock-8", "mock-9", "mock-10",
    "mock-11", "mock-12", "mock-13", "mock-14", "mock-15",
    "mock-16", "mock-17", "mock-18", "mock-19", "mock-20",
    "mock-21",
    "mock-err-1", "mock-err-2", "mock-err-3", "mock-err-4", "mock-err-5",
    "mock-err-6", "mock-err-7", "mock-err-8", "mock-err-9", "mock-err-10",
    "mock-err-11", "mock-err-12", "mock-err-13", "mock-err-14", "mock-err-15",
    "mock-err-16", "mock-err-17", "mock-err-18",
];

/// Lists all mock Profiles.
pub fn list_profiles() -> objectiveai::functions::profiles::response::ListProfile {
    objectiveai::functions::profiles::response::ListProfile {
        data: PROFILE_REPOSITORIES
            .iter()
            .map(|repo| objectiveai::functions::profiles::response::ListProfileItem {
                remote: objectiveai::functions::Remote::Mock,
                owner: "mock".to_string(),
                repository: repo.to_string(),
                commit: "mock".to_string(),
            })
            .collect(),
    }
}
