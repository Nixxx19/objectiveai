fn main() {
    #[cfg(feature = "viewer")]
    embed_viewer();
}

#[cfg(feature = "viewer")]
fn embed_viewer() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_dir = std::path::Path::new(&manifest_dir).parent().unwrap();
    let target = std::env::var("TARGET").unwrap();
    let profile = std::env::var("PROFILE").unwrap(); // "debug" or "release"

    // Run validate.sh with matching target and profile
    let validate_script = workspace_dir.join("objectiveai-viewer/validate.sh");
    let mut cmd = std::process::Command::new("bash");
    cmd.arg(&validate_script)
        .arg("--target")
        .arg(&target)
        .current_dir(workspace_dir);
    if profile == "release" {
        cmd.arg("--release");
    }
    let status = cmd.status().expect("Failed to run objectiveai-viewer/validate.sh");

    assert!(
        status.success(),
        "objectiveai-viewer/validate.sh failed. Run: bash objectiveai-viewer/build.sh --target {target}{}",
        if profile == "release" { " --release" } else { "" }
    );

    let binary_name = if target.contains("windows") {
        "objectiveai-viewer.exe"
    } else {
        "objectiveai-viewer"
    };

    let binary_path = workspace_dir
        .join("objectiveai-viewer")
        .join("embed")
        .join(&target)
        .join(&profile)
        .join(binary_name);

    println!(
        "cargo:rustc-env=OBJECTIVEAI_VIEWER_BINARY_PATH={}",
        binary_path.display()
    );
    println!("cargo:rerun-if-changed=../objectiveai-viewer/embed/");
}
