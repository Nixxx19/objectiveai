/// Re-invokes the current CLI as a subprocess with `--detach` removed from
/// the arguments. Emits the child PID, then forwards every JSON line the
/// child writes to its stdout — either to `handle` (if `Some`) or to this
/// process's own stdout (if `None`). Once `log_stream_ready` appears on
/// the child's stdout, exits with code 0; the orphan continues running
/// and writing more lines, but nobody reads them. If the child exits
/// without producing the handshake, forwards its exit code.
///
/// The orphan child has no idea about `handle` — it's a fresh CLI
/// invocation with the default `None` handle, writing JSONL to its own
/// stdout. The parent's forwarding loop is the place where the JSONL
/// stream gets routed.
pub async fn detach(handle: &objectiveai_cli_lib::output::Handle) -> ! {
    let exe = std::env::current_exe().expect("failed to get current executable path");
    let args: Vec<String> = std::env::args()
        .skip(1) // skip binary name
        .filter(|a| a != "--detach")
        .collect();

    let mut cmd = tokio::process::Command::new(exe);
    cmd.args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    // On Windows, create the child in a new process group and detach it from
    // the parent's job object so it survives when the parent exits.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }

    let mut child = cmd.spawn().expect("failed to spawn detached process");

    let pid = child.id().expect("failed to get child PID");
    objectiveai_cli_lib::output::Output::<objectiveai_cli_lib::output::Detached>::Notification(
        objectiveai_cli_lib::output::Detached { pid },
    )
    .emit(handle).await;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    let mut stdout_reader = tokio::io::BufReader::new(child_stdout);
    let mut stderr_reader = tokio::io::BufReader::new(child_stderr);
    let mut stdout_line = String::new();
    let mut stderr_line = String::new();
    let mut stdout_done = false;
    let mut stderr_done = false;

    loop {
        tokio::select! {
            result = tokio::io::AsyncBufReadExt::read_line(&mut stdout_reader, &mut stdout_line), if !stdout_done => {
                let n = result.unwrap_or(0);
                if n == 0 {
                    stdout_done = true;
                } else {
                    // Forward each line of the orphan's stdout to the
                    // parent's emission destination — `handle` if `Some`,
                    // else parent stdout. Keeps the JSONL stream
                    // consistent for whoever is consuming it.
                    match handle {
                        Some(stdin) => {
                            use tokio::io::AsyncWriteExt;
                            let mut guard = stdin.lock().await;
                            guard.write_all(stdout_line.as_bytes()).await
                                .expect("forward to child stdin failed");
                        }
                        None => {
                            print!("{stdout_line}");
                        }
                    }
                    if crate::log_line::parse_log_stream_ready(&stdout_line).is_some() {
                        std::process::exit(0);
                    }
                    stdout_line.clear();
                }
            }
            result = tokio::io::AsyncBufReadExt::read_line(&mut stderr_reader, &mut stderr_line), if !stderr_done => {
                let n = result.unwrap_or(0);
                if n == 0 {
                    stderr_done = true;
                } else {
                    eprint!("{stderr_line}");
                    stderr_line.clear();
                }
            }
        }
        if stdout_done && stderr_done {
            break;
        }
    }

    // Never saw the log line — child must have failed. Forward its exit code.
    let status = child.wait().await.expect("failed to wait for child");
    std::process::exit(status.code().unwrap_or(1))
}
