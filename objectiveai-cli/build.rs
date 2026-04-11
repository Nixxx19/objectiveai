fn main() {
    #[cfg(feature = "viewer")]
    build_viewer();
}

#[cfg(feature = "viewer")]
fn build_viewer() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_dir = std::path::Path::new(&manifest_dir).parent().unwrap();
    let profile = std::env::var("PROFILE").unwrap(); // "debug" or "release"

    let status = std::process::Command::new("cargo")
        .args(["build", "-p", "objectiveai-viewer"])
        .args(if profile == "release" {
            vec!["--release"]
        } else {
            vec![]
        })
        .current_dir(workspace_dir)
        .status()
        .expect("Failed to build objectiveai-viewer");

    assert!(status.success(), "objectiveai-viewer build failed");

    let binary_name = if cfg!(windows) {
        "objectiveai-viewer.exe"
    } else {
        "objectiveai-viewer"
    };

    let binary_path = workspace_dir
        .join("target")
        .join(if profile == "release" { "release" } else { "debug" })
        .join(binary_name);

    assert!(
        binary_path.exists(),
        "Expected objectiveai-viewer binary at {}",
        binary_path.display()
    );

    println!(
        "cargo:rustc-env=OBJECTIVEAI_VIEWER_BINARY_PATH={}",
        binary_path.display()
    );
    println!("cargo:rerun-if-changed=../objectiveai-viewer/src-tauri/src/");
    println!("cargo:rerun-if-changed=../objectiveai-viewer/src-tauri/Cargo.toml");
    println!("cargo:rerun-if-changed=../objectiveai-viewer/dist/");
}
