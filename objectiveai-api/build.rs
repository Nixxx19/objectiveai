fn main() {
    set_stack_size();

    #[cfg(feature = "orchestrator-bollard")]
    laboratories_local();
}

/// Set the main thread stack size to 16 MB for all supported platforms.
fn set_stack_size() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    let flag = match (os.as_str(), env.as_str()) {
        ("windows", "msvc") => "/STACK:16777216",
        ("windows", "gnu") => "-Wl,--stack,16777216",
        ("macos", _) | ("ios", _) => "-Wl,-stack_size,0x1000000",
        ("linux", _) | ("freebsd", _) | ("netbsd", _) | ("openbsd", _) | ("dragonfly", _) => {
            "-Wl,-z,stacksize=16777216"
        }
        _ => return,
    };

    println!("cargo:rustc-link-arg={flag}");
}

#[cfg(feature = "orchestrator-bollard")]
fn laboratories_local() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let musl_target = format!("{arch}-unknown-linux-musl");
    let profile = std::env::var("PROFILE").unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_dir = std::path::Path::new(&manifest_dir).parent().unwrap();

    // Run validate.sh with matching target and profile
    let validate_script = workspace_dir.join("objectiveai-mcp/validate.sh");
    let mut cmd = std::process::Command::new("bash");
    cmd.arg(&validate_script)
        .arg("--target")
        .arg(&musl_target)
        .current_dir(workspace_dir);
    if profile == "release" {
        cmd.arg("--release");
    }
    let status = cmd.status().expect("Failed to run objectiveai-mcp/validate.sh");

    assert!(
        status.success(),
        "objectiveai-mcp/validate.sh failed. Run: bash objectiveai-mcp/build.sh --target {musl_target}{}",
        if profile == "release" { " --release" } else { "" }
    );

    let binary_path = workspace_dir
        .join("objectiveai-mcp")
        .join("embed")
        .join(&musl_target)
        .join(&profile)
        .join("objectiveai-mcp");

    println!(
        "cargo:rustc-env=OBJECTIVEAI_MCP_BINARY_PATH={}",
        binary_path.display()
    );
    println!("cargo:rerun-if-changed=../objectiveai-mcp/embed/");
}
