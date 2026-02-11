//! Core expression types for JMESPath and Starlark evaluation.

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Result of an expression that may produce one or many values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// A single value.
    One(T),
    /// Multiple values (from array expressions).
    Many(Vec<T>),
}

/// An expression that can be either JMESPath or Starlark.
///
/// Serializes as `{"$jmespath": "..."}` or `{"$starlark": "..."}` in JSON.
///
/// # Examples
///
/// JMESPath:
/// ```json
/// {"$jmespath": "input.items[0].name"}
/// ```
///
/// Starlark:
/// ```json
/// {"$starlark": "input['items'][0]['name']"}
/// ```
#[derive(Debug, Clone)]
pub enum Expression {
    /// A JMESPath expression.
    JMESPath(String),
    /// A Starlark expression.
    Starlark(String),
}

impl Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Expression::JMESPath(expr) => map.serialize_entry("$jmespath", expr)?,
            Expression::Starlark(expr) => map.serialize_entry("$starlark", expr)?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct ExpressionVisitor;

        impl<'de> Visitor<'de> for ExpressionVisitor {
            type Value = Expression;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a map with exactly one key '$jmespath' or '$starlark' containing a string",
                )
            }

            fn visit_map<M>(self, mut map: M) -> Result<Expression, M::Error>
            where
                M: MapAccess<'de>,
            {
                // Get the first (and should be only) key
                let Some(key) = map.next_key::<String>()? else {
                    return Err(de::Error::custom(
                        "expected '$jmespath' or '$starlark' key, found empty map",
                    ));
                };

                // Get the string value
                let expr: String = map.next_value()?;

                // Ensure there are no more keys
                if map.next_key::<String>()?.is_some() {
                    return Err(de::Error::custom(
                        "expected exactly one expression key, found additional keys",
                    ));
                }

                match key.as_str() {
                    "$jmespath" => Ok(Expression::JMESPath(expr)),
                    "$starlark" => Ok(Expression::Starlark(expr)),
                    other => Err(de::Error::custom(format!(
                        "expected '$jmespath' or '$starlark', found '{}'",
                        other
                    ))),
                }
            }
        }

        deserializer.deserialize_map(ExpressionVisitor)
    }
}

impl Expression {
    /// Compiles the expression, allowing array results.
    ///
    /// Returns `OneOrMany::One` for single values or `OneOrMany::Many` for arrays.
    /// Null values are filtered out from array results.
    /// A Single Null value is treated as an empty array.
    pub fn compile_one_or_many<T>(
        &self,
        params: &super::Params,
    ) -> Result<OneOrMany<T>, super::ExpressionError>
    where
        T: DeserializeOwned,
    {
        let value = match self {
            Expression::JMESPath(jmespath) => {
                let expr = super::JMESPATH_RUNTIME.compile(jmespath)?;
                let value = expr.search(params)?;
                serde_json::to_value(value)?
            }
            Expression::Starlark(starlark) => super::starlark_eval(starlark, params)?,
        };
        Self::deserialize_result(value)
    }

    /// Deserialize expression result to the expected type.
    fn deserialize_result<T>(value: serde_json::Value) -> Result<OneOrMany<T>, super::ExpressionError>
    where
        T: DeserializeOwned,
    {
        let value: Option<OneOrMany<Option<T>>> = serde_json::from_value(value)
            .map_err(super::ExpressionError::DeserializationError)?;
        Ok(match value {
            Some(OneOrMany::One(Some(v))) => OneOrMany::One(v),
            Some(OneOrMany::One(None)) => OneOrMany::Many(Vec::new()),
            Some(OneOrMany::Many(mut vs)) => {
                vs.retain(|v| v.is_some());
                if vs.is_empty() {
                    OneOrMany::Many(Vec::new())
                } else if vs.len() == 1 {
                    OneOrMany::One(vs.into_iter().flatten().next().unwrap())
                } else {
                    OneOrMany::Many(vs.into_iter().flatten().collect())
                }
            }
            None => OneOrMany::Many(Vec::new()),
        })
    }

    /// Compiles the expression, expecting exactly one value.
    ///
    /// Accepts a single value or an array with exactly one element.
    /// Returns an error if the expression evaluates to null, an empty array,
    /// or an array with more than one element.
    pub fn compile_one<T>(
        &self,
        params: &super::Params,
    ) -> Result<T, super::ExpressionError>
    where
        T: DeserializeOwned,
    {
        let result = self.compile_one_or_many(params)?;
        match result {
            OneOrMany::One(value) => Ok(value),
            OneOrMany::Many(_) => Err(super::ExpressionError::ExpectedOneValueFoundMany),
        }
    }
}

/// A value that can be either a literal or an expression.
///
/// This allows Function definitions to mix static values with dynamic
/// expressions. During compilation, expressions are evaluated while
/// literal values pass through unchanged.
///
/// # Example
///
/// Literal value:
/// ```json
/// "hello world"
/// ```
///
/// JMESPath expression:
/// ```json
/// {"$jmespath": "input.greeting"}
/// ```
///
/// Starlark expression:
/// ```json
/// {"$starlark": "input['greeting']"}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WithExpression<T> {
    /// An expression (JMESPath or Starlark) to evaluate.
    Expression(Expression),
    /// A literal value.
    Value(T),
}

impl<T> std::default::Default for WithExpression<T>
where
    T: Default,
{
    fn default() -> Self {
        WithExpression::Value(T::default())
    }
}

impl<T> WithExpression<T>
where
    T: DeserializeOwned,
{
    /// Compiles the value, allowing array results from expressions.
    ///
    /// Literal values always return `OneOrMany::One`. Expressions may return
    /// either one or many values.
    pub fn compile_one_or_many(
        self,
        params: &super::Params,
    ) -> Result<OneOrMany<T>, super::ExpressionError> {
        match self {
            WithExpression::Expression(expr) => {
                expr.compile_one_or_many(params)
            }
            WithExpression::Value(value) => Ok(OneOrMany::One(value)),
        }
    }

    /// Compiles the value, expecting exactly one result.
    ///
    /// Literal values pass through unchanged. Expressions must evaluate to
    /// a single value or an array with exactly one element.
    pub fn compile_one(
        self,
        params: &super::Params,
    ) -> Result<T, super::ExpressionError> {
        match self {
            WithExpression::Expression(expr) => expr.compile_one(params),
            WithExpression::Value(value) => Ok(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::expression::{ExpressionError, Input, Params, ParamsOwned};
    use indexmap::IndexMap;

    fn empty_params() -> Params<'static, 'static, 'static> {
        Params::Owned(ParamsOwned {
            input: Input::Object(IndexMap::new()),
            output: None,
            map: None,
        })
    }

    fn params_with_object(
        pairs: Vec<(&str, Input)>,
    ) -> Params<'static, 'static, 'static> {
        let mut map = IndexMap::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), v);
        }
        Params::Owned(ParamsOwned {
            input: Input::Object(map),
            output: None,
            map: None,
        })
    }

    #[test]
    fn expression_compile_one_string_starlark() {
        let params = params_with_object(vec![(
            "name",
            Input::String("alice".to_string()),
        )]);
        let expr = Expression::Starlark("input['name']".to_string());

        let value: String = expr.compile_one(&params).unwrap();
        assert_eq!(value, "alice");
    }

    #[test]
    fn expression_compile_one_or_many_integers_from_array() {
        let params = params_with_object(vec![(
            "numbers",
            Input::Array(vec![
                Input::Integer(1),
                Input::Integer(2),
                Input::Integer(3),
            ]),
        )]);
        let expr = Expression::Starlark("input['numbers']".to_string());

        let result: OneOrMany<i64> = expr.compile_one_or_many(&params).unwrap();
        match result {
            OneOrMany::Many(values) => assert_eq!(values, vec![1, 2, 3]),
            other => panic!("expected Many variant, got {:?}", other),
        }
    }

    #[test]
    fn expression_compile_one_or_many_filters_nulls_and_empty_is_many() {
        let params = empty_params();

        // Single null becomes an empty Many
        let expr_null = Expression::Starlark("None".to_string());
        let result: OneOrMany<i64> = expr_null.compile_one_or_many(&params).unwrap();
        match result {
            OneOrMany::Many(values) => assert!(values.is_empty()),
            other => panic!("expected Many([]), got {:?}", other),
        }

        // Array with nulls filters them out
        let expr_array = Expression::Starlark("[1, None, 3]".to_string());
        let result: OneOrMany<i64> = expr_array.compile_one_or_many(&params).unwrap();
        match result {
            OneOrMany::Many(values) => assert_eq!(values, vec![1, 3]),
            other => panic!("expected Many([1, 3]), got {:?}", other),
        }
    }

    #[test]
    fn expression_compile_one_errors_when_many() {
        let params = empty_params();
        let expr = Expression::Starlark("[1, 2]".to_string());

        let err = expr.compile_one::<i64>(&params).unwrap_err();
        match err {
            ExpressionError::ExpectedOneValueFoundMany => {}
            other => panic!("expected ExpectedOneValueFoundMany, got {:?}", other),
        }
    }

    #[test]
    fn withexpression_literal_passthrough_one_and_one_or_many() {
        let params = empty_params();
        let with_expr = WithExpression::Value::<String>("hello".to_string());

        let one = with_expr.clone().compile_one(&params).unwrap();
        assert_eq!(one, "hello");

        let one_or_many = with_expr.compile_one_or_many(&params).unwrap();
        match one_or_many {
            OneOrMany::One(value) => assert_eq!(value, "hello"),
            other => panic!("expected One(\"hello\"), got {:?}", other),
        }
    }

    #[test]
    fn withexpression_expression_uses_underlying_expression_for_one_and_many() {
        // Use an input with a scalar and an array to exercise both paths.
        let params = params_with_object(vec![
            ("value", Input::Integer(42)),
            (
                "values",
                Input::Array(vec![
                    Input::Integer(1),
                    Input::Integer(2),
                    Input::Integer(3),
                ]),
            ),
        ]);

        // compile_one with scalar output
        let with_scalar =
            WithExpression::Expression::<i64>(Expression::Starlark("input['value']".to_string()));
        let scalar = with_scalar.compile_one(&params).unwrap();
        assert_eq!(scalar, 42);

        // compile_one_or_many with array output
        let with_many = WithExpression::Expression::<i64>(Expression::Starlark(
            "input['values']".to_string(),
        ));
        let result = with_many.compile_one_or_many(&params).unwrap();
        match result {
            OneOrMany::Many(values) => assert_eq!(values, vec![1, 2, 3]),
            other => panic!("expected Many([1, 2, 3]), got {:?}", other),
        }
    }
}

