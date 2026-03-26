//! Python execution module. Tries the system Python interpreter first
//! (when the `systempython` feature is enabled), then falls back to the
//! built-in RustPython interpreter (when the `rustpython` feature is enabled).
//!
//! The output of the Python code is expected to be valid JSON on stdout.
//! It is deserialized into the requested type `T`.

use std::path::Path;

/// Execute a Python script file and deserialize stdout as JSON into `T`.
pub fn exec_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, crate::error::Error> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| crate::error::Error::PythonFileRead(path.to_path_buf(), e))?;
    exec_code(&code)
}

/// Execute inline Python code and deserialize stdout as JSON into `T`.
pub fn exec_code<T: serde::de::DeserializeOwned>(code: &str) -> Result<T, crate::error::Error> {
    let stdout = exec_code_raw(code)?;
    let mut de = serde_json::Deserializer::from_str(&stdout);
    serde_path_to_error::deserialize(&mut de)
        .map_err(crate::error::Error::PythonDeserialize)
}

/// Execute inline Python code and return raw stdout.
fn exec_code_raw(code: &str) -> Result<String, crate::error::Error> {
    #[cfg(feature = "systempython")]
    if let Some(result) = try_system_python_code(code) {
        match result {
            Ok(stdout) => return Ok(stdout),
            Err(system_err) => {
                #[cfg(feature = "rustpython")]
                {
                    return exec_code_rustpython(code).or(Err(system_err));
                }
                #[cfg(not(feature = "rustpython"))]
                {
                    return Err(system_err);
                }
            }
        }
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
fn try_system_python_code(code: &str) -> Option<Result<String, crate::error::Error>> {
    use std::process::Command;
    let python = find_system_python()?;
    let output = Command::new(&python)
        .arg("-c")
        .arg(code)
        .output()
        .ok()?;
    if output.status.success() {
        Some(Ok(String::from_utf8_lossy(&output.stdout).into_owned()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Some(Err(crate::error::Error::PythonException(stderr)))
    }
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
fn exec_code_rustpython(code: &str) -> Result<String, crate::error::Error> {
    let interp = rustpython::InterpreterConfig::new()
        .init_stdlib()
        .interpreter();

    interp.enter(|vm| {
        let scope = vm.new_scope_with_builtins();
        match vm.run_code_string(scope, code, "<inline>".to_owned()) {
            Ok(_) => Ok(String::new()),
            Err(exc) => {
                let mut stderr = String::new();
                vm.write_exception(&mut stderr, &exc).ok();
                Err(crate::error::Error::PythonException(stderr))
            }
        }
    })
}
