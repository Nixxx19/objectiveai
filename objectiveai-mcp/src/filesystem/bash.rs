use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Debug, serde::Serialize)]
pub struct BashOutput {
    pub stdout: String,
    pub stderr: String,
    pub interrupted: bool,
}

/// Per-session shell state. Tracks CWD, shell snapshot, session env vars,
/// and tmux isolation across bash invocations.
#[derive(Debug, Clone)]
pub struct ShellState {
    /// Current working directory, persisted across commands.
    cwd: Arc<RwLock<PathBuf>>,
    /// Path to the shell environment snapshot file (functions, aliases, options).
    /// Captured once at session start from the user's shell config.
    snapshot_path: Arc<RwLock<Option<String>>>,
    /// Session-scoped environment variables (set via API, not from commands).
    session_env_vars: Arc<RwLock<HashMap<String, String>>>,
    /// Tmux socket env override. Set once tmux is first used.
    tmux_env: Arc<RwLock<Option<String>>>,
    /// Whether tmux has been used this session.
    tmux_used: Arc<RwLock<bool>>,
    /// The user's shell path (e.g., /bin/bash, /bin/zsh).
    shell_path: String,
}

impl ShellState {
    pub fn new() -> Self {
        let shell_path = detect_shell();
        Self {
            cwd: Arc::new(RwLock::new(
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            )),
            snapshot_path: Arc::new(RwLock::new(None)),
            session_env_vars: Arc::new(RwLock::new(HashMap::new())),
            tmux_env: Arc::new(RwLock::new(None)),
            tmux_used: Arc::new(RwLock::new(false)),
            shell_path,
        }
    }

    /// Initialize the shell snapshot asynchronously.
    /// Should be called once at session start.
    pub async fn init_snapshot(&self) {
        match create_shell_snapshot(&self.shell_path).await {
            Ok(path) => {
                *self.snapshot_path.write().unwrap() = Some(path);
            }
            Err(e) => {
                tracing::warn!("Failed to create shell snapshot: {e}");
            }
        }
    }

    pub fn get_cwd(&self) -> PathBuf {
        self.cwd.read().unwrap().clone()
    }

    fn set_cwd(&self, path: PathBuf) {
        *self.cwd.write().unwrap() = path;
    }

    pub fn set_session_env_var(&self, name: String, value: String) {
        self.session_env_vars.write().unwrap().insert(name, value);
    }

    pub fn delete_session_env_var(&self, name: &str) {
        self.session_env_vars.write().unwrap().remove(name);
    }

    pub fn get_session_env_vars(&self) -> HashMap<String, String> {
        self.session_env_vars.read().unwrap().clone()
    }

    fn get_snapshot_path(&self) -> Option<String> {
        self.snapshot_path.read().unwrap().clone()
    }

    fn mark_tmux_used(&self) {
        *self.tmux_used.write().unwrap() = true;
    }

    fn has_tmux_been_used(&self) -> bool {
        *self.tmux_used.read().unwrap()
    }

    fn get_tmux_env(&self) -> Option<String> {
        self.tmux_env.read().unwrap().clone()
    }

    fn set_tmux_env(&self, value: String) {
        *self.tmux_env.write().unwrap() = Some(value);
    }
}

pub async fn execute_bash(
    shell_state: &ShellState,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<String, String> {
    // Default 120s, max 600s
    let timeout_ms = timeout_ms.unwrap_or(120_000).min(600_000);
    let timeout_duration = Duration::from_millis(timeout_ms);

    // Track tmux usage
    if command.contains("tmux") {
        shell_state.mark_tmux_used();
        if shell_state.get_tmux_env().is_none() {
            if let Ok(socket_path) = init_tmux_socket().await {
                shell_state.set_tmux_env(socket_path);
            }
        }
    }

    // Build the full command string with all session state
    let cwd = shell_state.get_cwd();
    let snapshot_path = shell_state.get_snapshot_path();
    let has_snapshot = snapshot_path.is_some();

    let mut command_parts: Vec<String> = Vec::new();

    // 1. Source the shell snapshot (if available)
    if let Some(ref snap) = snapshot_path {
        command_parts.push(format!("source {} 2>/dev/null || true", shell_quote(snap)));
    }

    // 2. CD into saved CWD
    command_parts.push(format!("cd {}", shell_quote(&cwd.to_string_lossy())));

    // 3. The user's command (wrapped in eval for alias expansion)
    command_parts.push(format!("eval {}", shell_quote(command)));

    // 4. Save CWD after command execution
    let cwd_file = cwd_file_path();
    command_parts.push(format!("pwd -P >| {}", shell_quote(&cwd_file)));

    let command_string = command_parts.join(" && ");

    // Build spawn args: -c [-l] <command>
    // Use login shell (-l) only when no snapshot is available
    let mut args = vec!["-c".to_string()];
    if !has_snapshot {
        args.push("-l".to_string());
    }
    args.push(command_string);

    // Build environment overrides
    let mut env_overrides: HashMap<String, String> = shell_state.get_session_env_vars();

    // Tmux socket isolation
    if let Some(tmux_env) = shell_state.get_tmux_env() {
        env_overrides.insert("TMUX".into(), tmux_env);
    }

    let mut cmd = tokio::process::Command::new(&shell_state.shell_path);
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Apply session env var overrides
    for (key, value) in &env_overrides {
        cmd.env(key, value);
    }

    let child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    let output = match tokio::time::timeout(timeout_duration, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

            // Read the saved CWD from the temp file
            if let Ok(new_cwd) = std::fs::read_to_string(&cwd_file) {
                let new_cwd = new_cwd.trim();
                if !new_cwd.is_empty() {
                    shell_state.set_cwd(PathBuf::from(new_cwd));
                }
            }
            // Clean up the CWD file
            let _ = std::fs::remove_file(&cwd_file);

            BashOutput {
                stdout,
                stderr,
                interrupted: false,
            }
        }
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

/// Detect the user's shell from environment.
fn detect_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) {
            "bash".into()
        } else {
            "/bin/bash".into()
        }
    })
}

/// Generate a unique CWD temp file path for this invocation.
fn cwd_file_path() -> String {
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{}/objectiveai-mcp-{pid}-{ts}-cwd",
        std::env::temp_dir().to_string_lossy()
    )
}

/// Simple shell quoting for a single argument.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Create a shell snapshot by sourcing the user's shell config and capturing
/// functions, aliases, and options. Returns the path to the snapshot file.
async fn create_shell_snapshot(shell_path: &str) -> Result<String, String> {
    let shell_type = if shell_path.contains("zsh") {
        "zsh"
    } else {
        "bash"
    };

    let config_file = get_config_file(shell_path);

    // Config file is optional — snapshot still captures Claude Code defaults
    let has_config = std::path::Path::new(&config_file).exists();

    let snapshot_path = format!(
        "{}/objectiveai-mcp-snapshot-{}-{}.sh",
        std::env::temp_dir().to_string_lossy(),
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    );

    let source_line = if has_config {
        format!("source {} 2>/dev/null", shell_quote(&config_file))
    } else {
        "true".into()
    };

    let snapshot_script = if shell_type == "zsh" {
        format!(
            r#"
SNAPSHOT_FILE={snapshot}
{source}
{{
  echo '# Shell snapshot (zsh)'
  typeset -f 2>/dev/null
  alias 2>/dev/null | while IFS= read -r line; do echo "alias $line"; done
  setopt 2>/dev/null | while IFS= read -r opt; do echo "setopt $opt"; done
}} > "$SNAPSHOT_FILE" 2>/dev/null
"#,
            snapshot = shell_quote(&snapshot_path),
            source = source_line,
        )
    } else {
        format!(
            r#"
SNAPSHOT_FILE={snapshot}
{source}
{{
  echo '# Shell snapshot (bash)'
  declare -f 2>/dev/null
  alias 2>/dev/null | while IFS= read -r line; do echo "alias $line"; done
  shopt -p 2>/dev/null
}} > "$SNAPSHOT_FILE" 2>/dev/null
"#,
            snapshot = shell_quote(&snapshot_path),
            source = source_line,
        )
    };

    let result = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(shell_path)
            .args(["-c", &snapshot_script])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .map_err(|_| "Shell snapshot creation timed out".to_string())?
    .map_err(|e| format!("Failed to create snapshot: {e}"))?;

    if !std::path::Path::new(&snapshot_path).exists() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Snapshot file was not created: {stderr}"));
    }

    tracing::info!("Shell snapshot created at {snapshot_path}");
    Ok(snapshot_path)
}

/// Get the shell config file path based on shell type.
fn get_config_file(shell_path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".into());
    if shell_path.contains("zsh") {
        format!("{home}/.zshrc")
    } else {
        format!("{home}/.bashrc")
    }
}

/// Initialize an isolated tmux socket for this session.
async fn init_tmux_socket() -> Result<String, String> {
    let socket_path = format!(
        "{}/objectiveai-mcp-tmux-{}.sock",
        std::env::temp_dir().to_string_lossy(),
        std::process::id(),
    );

    let output = tokio::process::Command::new("tmux")
        .args(["-S", &socket_path, "new-session", "-d", "-s", "objectiveai"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to initialize tmux socket: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux initialization failed: {stderr}"));
    }

    tracing::info!("Tmux socket initialized at {socket_path}");
    Ok(socket_path)
}
