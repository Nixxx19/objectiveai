#[derive(Debug, PartialEq, serde::Deserialize)]
struct Foo {
    foo: String,
}

fn expected() -> Foo {
    Foo { foo: "bar".to_string() }
}

/// Bare dict literal, starlark-style — no print, just an expression.
#[test]
fn eval_dict_literal() {
    let result: Foo = crate::python::exec_code(r#"{"foo": "bar"}"#).unwrap();
    assert_eq!(result, expected());
}

/// Explicit print with json.dumps on a single line.
#[test]
fn print_dict() {
    let result: Foo = crate::python::exec_code(
        r#"import json; print(json.dumps({"foo": "bar"}))"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Defines main() that returns the dict, calls it inside if __name__ == "__main__".
#[test]
fn main_returns() {
    let result: Foo = crate::python::exec_code(r#"
import json

def main():
    return {"foo": "bar"}

if __name__ == "__main__":
    print(json.dumps(main()))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Dict printed inside if __name__ == "__main__" guard, starlark-style.
#[test]
fn starlark_style_in_main_guard() {
    let result: Foo = crate::python::exec_code(r#"
import json

if __name__ == "__main__":
    print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Defines main(), prints its return value at top level.
#[test]
fn print_main_return() {
    let result: Foo = crate::python::exec_code(r#"
import json

def main():
    return {"foo": "bar"}

print(json.dumps(main()))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Unused function definition followed by a print.
#[test]
fn unused_fn_then_eval() {
    let result: Foo = crate::python::exec_code(r#"
import json

def add(a, b):
    return a + b

print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Another unused function definition followed by a print (same pattern, different name).
#[test]
fn unused_fn_then_print() {
    let result: Foo = crate::python::exec_code(r#"
import json

def add(a, b):
    return a + b

print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Uses sys.stdout.write instead of print — no trailing newline.
#[test]
fn sys_stdout_write() {
    let result: Foo = crate::python::exec_code(r#"
import json, sys
sys.stdout.write(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Defines a function and calls it as a bare expression (no print).
#[test]
fn bare_function_call() {
    let result: Foo = crate::python::exec_code(r#"
def get():
    return {"foo": "bar"}

get()
"#).unwrap();
    assert_eq!(result, expected());
}

/// Assigns to a variable, then references it as a bare expression.
#[test]
fn assign_then_bare_variable() {
    let result: Foo = crate::python::exec_code(r#"
x = {"foo": "bar"}
x
"#).unwrap();
    assert_eq!(result, expected());
}

/// Dict literal split across multiple lines with trailing comma.
#[test]
fn multiline_dict_expression() {
    let result: Foo = crate::python::exec_code(r#"
{
    "foo": "bar",
}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Dict comprehension as a bare expression.
#[test]
fn dict_comprehension() {
    let result: Foo = crate::python::exec_code(
        r#"{k: v for k, v in [("foo", "bar")]}"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Ternary/conditional expression returning a dict.
#[test]
fn ternary_expression() {
    let result: Foo = crate::python::exec_code(
        r#"{"foo": "bar"} if True else None"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Walrus operator (:=) as a bare expression.
#[test]
fn walrus_operator() {
    let result: Foo = crate::python::exec_code(
        r#"(x := {"foo": "bar"})"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Immediately-invoked lambda returning a dict.
#[test]
fn lambda_call() {
    let result: Foo = crate::python::exec_code(
        r#"(lambda: {"foo": "bar"})()"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Two statements on one line separated by semicolon, last is a bare expression.
#[test]
fn semicolons_one_line() {
    let result: Foo = crate::python::exec_code(
        r#"x = 1; {"foo": "bar"}"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Uses dict() constructor instead of literal syntax.
#[test]
fn nested_function_call_dict() {
    let result: Foo = crate::python::exec_code(
        r#"dict(foo="bar")"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Prints debug info to stderr, then has a bare expression on the last line.
#[test]
fn stderr_debug_then_bare_expression() {
    let result: Foo = crate::python::exec_code(r#"
import sys
print("debug info", file=sys.stderr)
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Prints unrelated text to stdout, then has a bare expression as the last line.
/// The wrapper should use the eval'd expression, not the printed noise.
#[test]
fn stdout_noise_then_bare_expression() {
    let result: Foo = crate::python::exec_code(r#"
print("some random debug output")
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Defines a class with a method, calls it as a bare expression.
#[test]
fn class_method_call() {
    let result: Foo = crate::python::exec_code(r#"
class C:
    def get(self):
        return {"foo": "bar"}

C().get()
"#).unwrap();
    assert_eq!(result, expected());
}

/// Prints JSON with end="" (no trailing newline).
#[test]
fn print_no_newline() {
    let result: Foo = crate::python::exec_code(
        r#"import json; print(json.dumps({"foo": "bar"}), end="")"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Multiple bare expressions — only the last one is captured by the harness.
#[test]
fn multiple_bare_expressions_last_wins() {
    let result: Foo = crate::python::exec_code(r#"
1
2
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Bare expression followed by trailing blank lines.
#[test]
fn trailing_blank_lines() {
    let result: Foo = crate::python::exec_code("{\"foo\": \"bar\"}\n\n\n").unwrap();
    assert_eq!(result, expected());
}

/// Bare expression followed by a trailing comment.
#[test]
fn trailing_comment_after_expression() {
    let result: Foo = crate::python::exec_code("{\"foo\": \"bar\"}\n# done").unwrap();
    assert_eq!(result, expected());
}

/// Expression followed by a trailing `pass` statement — expression is no longer last,
/// so the user must print explicitly.
#[test]
fn trailing_pass_after_expression() {
    let result: Foo = crate::python::exec_code(r#"
import json
print(json.dumps({"foo": "bar"}))
pass
"#).unwrap();
    assert_eq!(result, expected());
}

/// User code defines `_json`, which collides with the wrapper's internal variable.
/// The wrapper runs in a separate scope so this should not interfere.
#[test]
fn user_defines_underscore_json() {
    let result: Foo = crate::python::exec_code(r#"
import json
_json = None
print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// User code defines `_capture`, which collides with the wrapper's internal variable.
#[test]
fn user_defines_underscore_capture() {
    let result: Foo = crate::python::exec_code(r#"
import json
_capture = "oops"
print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// User code deletes the `sys` module after importing it.
#[test]
fn user_deletes_sys() {
    let result: Foo = crate::python::exec_code(r#"
import json, sys
del sys
print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Variable defined on one line, used in a bare expression on the next.
#[test]
fn global_variable_in_expression() {
    let result: Foo = crate::python::exec_code(r#"
x = "bar"
{"foo": x}
"#).unwrap();
    assert_eq!(result, expected());
}

/// User code itself calls exec() to define a variable, then prints it.
#[test]
fn nested_exec_in_user_code() {
    let result: Foo = crate::python::exec_code(r#"
import json
exec('result = {"foo": "bar"}')
print(json.dumps(result))
"#).unwrap();
    assert_eq!(result, expected());
}

/// User code reassigns sys.stdout to a StringIO, then restores it and prints.
#[test]
fn user_reassigns_stdout_then_prints() {
    let result: Foo = crate::python::exec_code(r#"
import json, sys, io
sys.stdout = io.StringIO()
sys.stdout = sys.__stdout__
print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Prints random garbage before printing the correct JSON on the last line.
/// The captured stdout is the concatenation of both prints, which is not
/// valid JSON for Foo. This should fail to deserialize.
#[test]
/// Code that raises an exception, catches it, then returns via bare expression.
#[test]
fn try_except_then_bare_expression() {
    let result: Foo = crate::python::exec_code(r#"
try:
    raise ValueError("oops")
except:
    pass
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Code that uses __name__ inside an if guard — verifies __name__ == "__main__" in exec().
#[test]
fn name_equals_main_in_exec() {
    let result: Foo = crate::python::exec_code(r#"
import json
if __name__ == "__main__":
    print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Code that redefines print, then uses the original via builtins.
#[test]
fn redefine_print_use_builtins() {
    let result: Foo = crate::python::exec_code(r#"
import json, builtins
print = lambda *a: None
builtins.print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Bare expression after a for loop.
#[test]
fn bare_expression_after_for_loop() {
    let result: Foo = crate::python::exec_code(r#"
items = []
for i in range(3):
    items.append(i)
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Bare expression after a with statement.
#[test]
fn bare_expression_after_with() {
    let result: Foo = crate::python::exec_code(r#"
import io
with io.StringIO() as f:
    f.write("ignored")
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Bare expression after try/except/finally.
#[test]
fn bare_expression_after_try_finally() {
    let result: Foo = crate::python::exec_code(r#"
x = None
try:
    x = 1
finally:
    x = 2
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// User code sets a variable with the exact __oai_ prefix the harness uses.
#[test]
fn user_sets_oai_prefix_variable() {
    let result: Foo = crate::python::exec_code(r#"
import json
__oai_capture = "sabotage"
__oai_json = None
__oai_result = 12345
print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Code that uses globals() to set a variable, then references it in bare expression.
#[test]
fn globals_dict_then_bare_expression() {
    let result: Foo = crate::python::exec_code(r#"
globals()["x"] = "bar"
{"foo": x}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Code that uses __import__ for dynamic import.
#[test]
fn dynamic_import() {
    let result: Foo = crate::python::exec_code(r#"
json = __import__("json")
print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Deeply nested dict as bare expression.
#[test]
fn deeply_nested_dict() {
    let result: serde_json::Value = crate::python::exec_code(r#"
{"a": {"b": {"c": {"foo": "bar"}}}}
"#).unwrap();
    assert_eq!(result["a"]["b"]["c"]["foo"], "bar");
}

/// Multiline string containing what looks like Python code, followed by bare expression.
#[test]
fn multiline_string_then_bare_expression() {
    let result: Foo = crate::python::exec_code(r#"
code = """
def fake():
    return {"wrong": "value"}
print("this is not executed")
"""
{"foo": "bar"}
"#).unwrap();
    assert_eq!(result, expected());
}

/// Unicode and emoji in the dict value.
#[test]
fn unicode_emoji_value() {
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Uni { msg: String }
    let result: Uni = crate::python::exec_code(r#"
{"msg": "hello 🦀 world àéîõü"}
"#).unwrap();
    assert_eq!(result, Uni { msg: "hello 🦀 world àéîõü".to_string() });
}

/// Code that uses *args and **kwargs, then bare expression.
#[test]
fn args_kwargs_then_bare_expression() {
    let result: Foo = crate::python::exec_code(r#"
def make(*args, **kwargs):
    return kwargs

make("ignored", foo="bar")
"#).unwrap();
    assert_eq!(result, expected());
}

/// Code that uses type() to dynamically create a class, then calls a method.
#[test]
fn dynamic_class_creation() {
    let result: Foo = crate::python::exec_code(r#"
MyClass = type("MyClass", (), {"get": lambda self: {"foo": "bar"}})
MyClass().get()
"#).unwrap();
    assert_eq!(result, expected());
}

/// List comprehension producing a single-element list, indexed to get the dict.
#[test]
fn list_comprehension_indexed() {
    let result: Foo = crate::python::exec_code(r#"
[{"foo": v} for v in ["bar"]][0]
"#).unwrap();
    assert_eq!(result, expected());
}

/// Code that uses `exit()` guard — doesn't actually exit because we use the value before.
#[test]
fn conditional_with_no_exit() {
    let result: Foo = crate::python::exec_code(r#"
import json
should_exit = False
if should_exit:
    exit(1)
print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

#[test]
fn garbage_stdout_before_correct_print() {
    let err = crate::python::exec_code::<Foo>(r#"
import json
print("here is some random garbage!!!")
print(json.dumps({"foo": "bar"}))
"#).unwrap_err();
    assert!(matches!(err, crate::error::Error::PythonDeserialize(_)));
}
