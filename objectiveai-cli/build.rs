fn main() {
    #[cfg(feature = "laboratory")]
    laboratory();
}

#[cfg(feature = "laboratory")]
fn laboratory() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let musl_target = format!("{arch}-unknown-linux-musl");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_dir = std::path::Path::new(&manifest_dir).parent().unwrap();

    let status = std::process::Command::new("cargo")
        .args(["build", "--release", "--target", &musl_target])
        .args(["--no-default-features", "--features", "filesystem"])
        .args(["-p", "objectiveai-mcp"])
        .current_dir(workspace_dir)
        .status()
        .expect("Failed to build objectiveai-mcp for linux-musl target");

    assert!(
        status.success(),
        "objectiveai-mcp build for {musl_target} failed"
    );

    let binary_path = workspace_dir
        .join("target")
        .join(&musl_target)
        .join("release")
        .join("objectiveai-mcp");

    assert!(
        binary_path.exists(),
        "Expected objectiveai-mcp binary at {}",
        binary_path.display()
    );

    println!(
        "cargo:rustc-env=OBJECTIVEAI_MCP_BINARY_PATH={}",
        binary_path.display()
    );
    println!("cargo:rerun-if-changed=../objectiveai-mcp/src/");
    println!("cargo:rerun-if-changed=../objectiveai-mcp/Cargo.toml");
}
