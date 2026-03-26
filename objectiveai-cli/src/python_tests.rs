#[derive(Debug, PartialEq, serde::Deserialize)]
struct Foo {
    foo: String,
}

const EXPECTED: Foo = Foo { foo: String::new() };

fn expected() -> Foo {
    Foo { foo: "bar".to_string() }
}

/// Single expression, starlark-style: just a dict literal.
#[test]
fn eval_dict_literal() {
    let result: Foo = crate::python::exec_code(r#"{"foo": "bar"}"#).unwrap();
    assert_eq!(result, expected());
}

/// Prints the dict as JSON.
#[test]
fn print_dict() {
    let result: Foo = crate::python::exec_code(
        r#"import json; print(json.dumps({"foo": "bar"}))"#
    ).unwrap();
    assert_eq!(result, expected());
}

/// Defines main() that returns the dict, calls it from if __name__ == "__main__".
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

/// Starlark-style dict literal nested inside if __name__ == "__main__".
#[test]
fn starlark_style_in_main_guard() {
    let result: Foo = crate::python::exec_code(r#"
import json

if __name__ == "__main__":
    print(json.dumps({"foo": "bar"}))
"#).unwrap();
    assert_eq!(result, expected());
}

/// Defines main() that returns the dict, prints its return value.
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

/// Single expression with an unused function definition above it.
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

/// Prints the dict with an unused function definition above it.
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
