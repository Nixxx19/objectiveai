/// Re-invokes the current CLI as a subprocess with `--detach` removed from
/// the arguments. Prints the child PID, then forwards all stdout/stderr.
/// Once the log availability line appears on stdout, exits with code 0 —
/// the child continues as an orphan. If the child exits without printing
/// the log line, forwards its exit code.
pub async fn detach() -> ! {
    let exe = std::env::current_exe().expect("failed to get current executable path");
    let args: Vec<String> = std::env::args()
        .skip(1) // skip binary name
        .filter(|a| a != "--detach")
        .collect();

    let mut child = tokio::process::Command::new(exe)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn detached process");

    let pid = child.id().expect("failed to get child PID");
    println!("PID: {pid}");

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
                    print!("{stdout_line}");
                    if crate::log_line::is_log_id_line(&stdout_line) {
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
