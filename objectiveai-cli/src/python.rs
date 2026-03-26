//! Python execution module. Tries the system Python interpreter first
//! (when the `systempython` feature is enabled), then falls back to the
//! built-in RustPython interpreter (when the `rustpython` feature is enabled).

use std::path::Path;

/// Result of executing Python code.
pub struct PythonOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Execute a Python script file.
pub fn exec_file(path: &Path, args: &[&str]) -> Result<PythonOutput, crate::error::Error> {
    let _ = args;
    let code = std::fs::read_to_string(path)
        .map_err(|e| crate::error::Error::PythonFileRead(path.to_path_buf(), e))?;
    exec_code(&code)
}

/// Execute an inline Python code string.
pub fn exec_code(code: &str) -> Result<PythonOutput, crate::error::Error> {
    #[cfg(feature = "systempython")]
    if let Some(output) = try_system_python_code(code) {
        return Ok(output);
    }
    #[cfg(feature = "rustpython")]
    {
        return exec_code_rustpython(code);
    }
    #[cfg(not(feature = "rustpython"))]
    {
        let _ = code;
        Err(crate::error::Error::PythonNotFound)
    }
}

#[cfg(feature = "systempython")]
fn try_system_python_code(code: &str) -> Option<PythonOutput> {
    use std::process::Command;
    let python = find_system_python()?;
    let output = Command::new(&python)
        .arg("-c")
        .arg(code)
        .output()
        .ok()?;
    Some(PythonOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

#[cfg(feature = "systempython")]
fn find_system_python() -> Option<String> {
    use std::process::Command;
    for name in &["python3", "python", "py"] {
        if Command::new(name).arg("--version").output().is_ok() {
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(feature = "rustpython")]
fn exec_code_rustpython(code: &str) -> Result<PythonOutput, crate::error::Error> {
    let interp = rustpython::InterpreterConfig::new()
        .init_stdlib()
        .interpreter();

    let result = interp.enter(|vm| {
        let scope = vm.new_scope_with_builtins();
        match vm.run_code_string(scope, code, "<inline>".to_owned()) {
            Ok(_) => PythonOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
            Err(exc) => {
                let mut stderr = String::new();
                vm.write_exception(&mut stderr, &exc).ok();
                PythonOutput {
                    stdout: String::new(),
                    stderr,
                    success: false,
                }
            }
        }
    });

    Ok(result)
}
