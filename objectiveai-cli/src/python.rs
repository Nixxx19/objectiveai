//! Python execution module. Tries the system Python interpreter first
//! (when the `systempython` feature is enabled), then falls back to the
//! built-in RustPython interpreter (when the `rustpython` feature is enabled).
//!
//! User code is wrapped in a harness that uses `ast` to detect bare trailing
//! expressions. The harness outputs a JSON envelope with `eval` (the expression
//! result, or null) and `stdout` (captured print output). The Rust side tries
//! to deserialize `eval` first, falling back to `stdout`.

use std::path::Path;

/// The JSON envelope produced by the Python harness.
#[derive(serde::Deserialize)]
struct HarnessOutput {
    eval: serde_json::Value,
    stdout: String,
}

/// Execute a Python script file and deserialize the output as JSON into `T`.
pub fn exec_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, crate::error::Error> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| crate::error::Error::PythonFileRead(path.to_path_buf(), e))?;
    exec_code(&code)
}

/// Execute inline Python code and deserialize the output as JSON into `T`.
///
/// The harness wraps user code and produces `{"eval": ..., "stdout": "..."}`.
/// If `eval` is non-null, we try to deserialize it into `T`.
/// Otherwise, we deserialize the raw `stdout` string into `T`.
pub fn exec_code<T: serde::de::DeserializeOwned>(code: &str) -> Result<T, crate::error::Error> {
    let raw = exec_code_raw(code)?;
    let envelope: HarnessOutput = serde_json::from_str(&raw)
        .map_err(|e| crate::error::Error::PythonHarnessBroken(e.to_string()))?;

    // Try eval first (non-null means the last statement was a bare expression)
    let eval_err = if !envelope.eval.is_null() {
        let eval_str = envelope.eval.to_string();
        let mut de = serde_json::Deserializer::from_str(&eval_str);
        match serde_path_to_error::deserialize(&mut de) {
            Ok(result) => return Ok(result),
            Err(e) => Some(e),
        }
    } else {
        None
    };

    // Fall back to stdout (from print() calls)
    let mut de = serde_json::Deserializer::from_str(&envelope.stdout);
    match serde_path_to_error::deserialize(&mut de) {
        Ok(result) => Ok(result),
        Err(stdout_err) => Err(crate::error::Error::PythonDeserialize(
            eval_err.unwrap_or(stdout_err)
        )),
    }
}

/// Wraps user code in a harness that outputs a JSON envelope with eval and stdout.
fn wrap_code(code: &str) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(code);
    format!(
        r#"
import ast as __oai_ast, json as __oai_json, sys as __oai_sys, io as __oai_io, base64 as __oai_b64
__oai_code = __oai_b64.b64decode("{}").decode()
__oai_tree = __oai_ast.parse(__oai_code)
__oai_capture = __oai_io.StringIO()
__oai_old_stdout = __oai_sys.stdout
__oai_old_dunder_stdout = __oai_sys.__stdout__
__oai_sys.stdout = __oai_capture
__oai_sys.__stdout__ = __oai_capture
__oai_print = print
__oai_eval = None
if __oai_tree.body and isinstance(__oai_tree.body[-1], __oai_ast.Expr):
    __oai_last = __oai_tree.body.pop()
    exec(compile(__oai_tree, "<inline>", "exec"))
    __oai_eval = eval(compile(__oai_ast.Expression(__oai_last.value), "<inline>", "eval"))
else:
    exec(compile(__oai_tree, "<inline>", "exec"))
__oai_stdout = __oai_capture.getvalue()
__oai_sys.stdout = __oai_old_stdout
__oai_sys.__stdout__ = __oai_old_dunder_stdout
__oai_print(__oai_json.dumps({{"eval": __oai_eval, "stdout": __oai_stdout}}))
"#,
        encoded
    )
}

/// Execute inline Python code and return raw stdout from the harness.
fn exec_code_raw(code: &str) -> Result<String, crate::error::Error> {
    let wrapped = wrap_code(code);
    #[cfg(feature = "systempython")]
    if let Some(result) = try_system_python_code(&wrapped) {
        match result {
            Ok(stdout) => return Ok(stdout),
            Err(system_err) => {
                #[cfg(feature = "rustpython")]
                {
                    return exec_code_rustpython(&wrapped).or(Err(system_err));
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
        return exec_code_rustpython(&wrapped);
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
