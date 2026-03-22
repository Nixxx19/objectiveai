use std::collections::HashSet;

#[test]
fn no_duplicate_names_across_listers() {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    let agents = super::list_agents();
    for path in &agents.data {
        if !seen.insert(path.name().to_string()) {
            duplicates.push(path.name().to_string());
        }
    }

    let swarms = super::list_swarms();
    for path in &swarms.data {
        if !seen.insert(path.name().to_string()) {
            duplicates.push(path.name().to_string());
        }
    }

    let functions = super::list_functions();
    for path in &functions.data {
        if !seen.insert(path.name().to_string()) {
            duplicates.push(path.name().to_string());
        }
    }

    let profiles = super::list_profiles();
    for path in &profiles.data {
        if !seen.insert(path.name().to_string()) {
            duplicates.push(path.name().to_string());
        }
    }

    assert!(
        duplicates.is_empty(),
        "duplicate names across mock listers: {:?}",
        duplicates,
    );
}
