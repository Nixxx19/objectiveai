use std::process::Stdio;
use std::time::Duration;

#[derive(Debug, serde::Serialize)]
pub struct BashOutput {
    pub stdout: String,
    pub stderr: String,
    pub interrupted: bool,
}

pub async fn execute_bash(
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<String, String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    // Default 120s, max 600s
    let timeout_ms = timeout_ms.unwrap_or(120_000).min(600_000);
    let timeout_duration = Duration::from_millis(timeout_ms);

    let child = tokio::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    let output = match tokio::time::timeout(timeout_duration, child.wait_with_output()).await {
        Ok(Ok(output)) => BashOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            interrupted: false,
        },
        Ok(Err(e)) => return Err(format!("Command failed: {e}")),
        Err(_) => BashOutput {
            stdout: String::new(),
            stderr: format!("Command timed out after {timeout_ms}ms"),
            interrupted: true,
        },
    };

    serde_json::to_string_pretty(&output)
        .map_err(|e| format!("Failed to serialize output: {e}"))
}
