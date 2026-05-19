//! Coverage test: every `.route(...)` in `objectiveai-api/src/run.rs` must
//! have a matching leaf file under `objectiveai-cli/src/api/...`.
//!
//! Convention: `<METHOD> /<a>/<b>/<c>` maps to
//! `objectiveai-cli/src/api/<a>/<b>/<c>/<method>.rs`, where `<method>` is the
//! lowercased HTTP verb (`post`, `get`, `delete`).

#[test]
fn every_api_route_has_a_cli_leaf() {
    let run_rs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("objectiveai-api")
        .join("src")
        .join("run.rs");
    let src = std::fs::read_to_string(&run_rs_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", run_rs_path.display()));

    // Captures every `.route("<path>", axum::routing::<method>(...)` declaration.
    let re = regex::Regex::new(
        r#"\.route\(\s*"([^"]+)"\s*,\s*(?:axum::routing::)?(get|post|delete|put)\("#,
    )
    .expect("regex compiles");

    let cli_src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut missing: Vec<String> = Vec::new();

    for cap in re.captures_iter(&src) {
        let path = cap.get(1).unwrap().as_str();
        let method = cap.get(2).unwrap().as_str();
        let segments: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // <cli_src_dir>/api/<...segments>/<method>.rs
        let mut rel = std::path::PathBuf::from("api");
        for seg in &segments {
            rel.push(seg);
        }
        rel.push(format!("{method}.rs"));

        let abs = cli_src_dir.join(&rel);
        if !abs.exists() {
            missing.push(format!("{} {} -> src/{}", method.to_uppercase(), path, rel.display()));
        }
    }

    assert!(
        missing.is_empty(),
        "uncovered API endpoints (each route in objectiveai-api/src/run.rs needs a matching CLI leaf):\n  {}",
        missing.join("\n  "),
    );
}
