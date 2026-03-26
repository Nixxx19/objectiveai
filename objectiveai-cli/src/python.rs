//! Python execution module. Tries the system Python interpreter first,
//! falls back to the built-in RustPython interpreter when the `rustpython`
//! feature is enabled.

use std::path::Path;
use std::process::Command;

/// Result of executing Python code.
pub struct PythonOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Execute a Python script file.
pub fn exec_file(path: &Path, args: &[&str]) -> Result<PythonOutput, crate::error::Error> {
    if let Some(output) = try_system_python_file(path, args) {
        return Ok(output);
    }
    #[cfg(feature = "rustpython")]
    {
        return exec_file_rustpython(path);
    }
    #[cfg(not(feature = "rustpython"))]
    {
        Err(crate::error::Error::PythonNotFound)
    }
}

/// Execute an inline Python code string.
pub fn exec_code(code: &str) -> Result<PythonOutput, crate::error::Error> {
    if let Some(output) = try_system_python_code(code) {
        return Ok(output);
    }
    #[cfg(feature = "rustpython")]
    {
        return exec_code_rustpython(code);
    }
    #[cfg(not(feature = "rustpython"))]
    {
        Err(crate::error::Error::PythonNotFound)
    }
}

/// Try to find and run a Python file using the system interpreter.
fn try_system_python_file(path: &Path, args: &[&str]) -> Option<PythonOutput> {
    let python = find_system_python()?;
    let output = Command::new(&python)
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    Some(PythonOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

/// Try to run inline Python code using the system interpreter.
fn try_system_python_code(code: &str) -> Option<PythonOutput> {
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

/// Find a system Python interpreter (python3, python, py).
fn find_system_python() -> Option<String> {
    for name in &["python3", "python", "py"] {
        if Command::new(name).arg("--version").output().is_ok() {
            return Some(name.to_string());
        }
    }
    None
}

/// Execute a Python file using the built-in RustPython interpreter.
#[cfg(feature = "rustpython")]
fn exec_file_rustpython(path: &Path) -> Result<PythonOutput, crate::error::Error> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| crate::error::Error::PythonFileRead(path.to_path_buf(), e))?;
    exec_code_rustpython(&code)
}

/// Execute inline Python code using the built-in RustPython interpreter.
#[cfg(feature = "rustpython")]
fn exec_code_rustpython(code: &str) -> Result<PythonOutput, crate::error::Error> {
    use rustpython::vm::Interpreter;

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
