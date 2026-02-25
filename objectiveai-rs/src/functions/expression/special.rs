//! Special predefined expression variants.

use serde::{Deserialize, Serialize};

/// Predefined expression behaviors that require no user-authored code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Special {
    /// Returns the params input as-is.
    Input,
    /// Returns the params output as-is.
    Output,
    /// L1-normalizes vector output. If output is `Function(Vector)` or
    /// `MapFunction(Vec<Scalar>)`, returns L1-normalized `FunctionOutput::Vector`.
    /// Otherwise returns the output as-is.
    L1NormalizedFunctionOutput,
    /// Returns the length of input['items'] as u64
    InputItemsOutputLength,
    /// Splits an input containing items and optionally context into multiple inputs
    InputItemsOptionalContextSplit,
    /// Merges multiple inputs containing items and optionally context into a single input
    InputItemsOptionalContextMerge,
    /// Returns the scores from vector completion output, otherwise returns the output as-is.
    VectorCompletionScores,
    /// Returns the scores from vector completion output, summed, each multiplied by a corresponding weight, the first weighted at 0, the final weighted at 1, and the rest weighted evenly in between, otherwise returns the output as-is.
    VectorCompletionScoresWeightedSum,
}

/// Trait for types that can be produced from a [`Special`] expression variant.
pub trait FromSpecial: Sized {
    fn from_special(
        special: &Special,
        params: &super::Params,
    ) -> Result<Self, super::ExpressionError>;
}

/// Macro for types that never support any Special variant.
macro_rules! impl_from_special_unsupported {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::functions::expression::FromSpecial for $ty {
                fn from_special(
                    _special: &$crate::functions::expression::Special,
                    _params: &$crate::functions::expression::Params,
                ) -> Result<Self, $crate::functions::expression::ExpressionError> {
                    Err($crate::functions::expression::ExpressionError::UnsupportedSpecial)
                }
            }
        )+
    };
}
pub(crate) use impl_from_special_unsupported;

impl_from_special_unsupported!(bool, i64, String);

impl<K, V, S> FromSpecial for indexmap::IndexMap<K, V, S>
where
    K: Sized,
    V: Sized,
    S: Sized,
{
    fn from_special(
        _special: &Special,
        _params: &super::Params,
    ) -> Result<Self, super::ExpressionError> {
        Err(super::ExpressionError::UnsupportedSpecial)
    }
}

impl FromSpecial for u64 {
    fn from_special(
        special: &Special,
        params: &super::Params,
    ) -> Result<Self, super::ExpressionError> {
        match special {
            Special::InputItemsOutputLength => {
                let input = match params {
                    super::Params::Owned(o) => &o.input,
                    super::Params::Ref(r) => r.input,
                };
                match input {
                    super::Input::Object(map) => match map.get("items") {
                        Some(super::Input::Array(arr)) => Ok(arr.len() as u64),
                        _ => Err(super::ExpressionError::UnsupportedSpecial),
                    },
                    _ => Err(super::ExpressionError::UnsupportedSpecial),
                }
            }
            _ => Err(super::ExpressionError::UnsupportedSpecial),
        }
    }
}

impl<T: FromSpecial> FromSpecial for super::OneOrMany<T> {
    fn from_special(
        special: &Special,
        params: &super::Params,
    ) -> Result<Self, super::ExpressionError> {
        Ok(super::OneOrMany::One(T::from_special(special, params)?))
    }
}

impl<T: FromSpecial> FromSpecial for Option<T> {
    fn from_special(
        special: &Special,
        params: &super::Params,
    ) -> Result<Self, super::ExpressionError> {
        Ok(Some(T::from_special(special, params)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::expression::{
        ExpressionError, FunctionOutput, Input, InputExpression, Params,
        ParamsOwned, TaskOutputOwned, VectorCompletionOutput,
    };
    use crate::vector::completions::response::Vote;
    use indexmap::IndexMap;
    use rust_decimal::dec;

    fn make_vote() -> Vote {
        Vote {
            model: "openai/gpt-4o".to_string(),
            ensemble_index: 0,
            flat_ensemble_index: 0,
            prompt_id: "p1".to_string(),
            tools_id: None,
            responses_ids: vec!["r1".to_string(), "r2".to_string()],
            vote: vec![dec!(1), dec!(0)],
            weight: dec!(1),
            retry: None,
            from_cache: None,
            from_rng: None,
            completion_index: None,
        }
    }

    // ── Special::Input ──────────────────────────────────────────────────

    #[test]
    fn special_input_returns_string_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::String("hello".to_string()),
            output: None,
            map: None,
        });
        let result = Input::from_special(&Special::Input, &params).unwrap();
        assert_eq!(result, Input::String("hello".to_string()));
    }

    #[test]
    fn special_input_returns_object_input() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Input::String("alice".to_string()));
        obj.insert("age".to_string(), Input::Integer(30));
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj.clone()),
            output: None,
            map: None,
        });
        let result = Input::from_special(&Special::Input, &params).unwrap();
        assert_eq!(result, Input::Object(obj));
    }

    #[test]
    fn special_input_returns_input_expression() {
        let params = Params::Owned(ParamsOwned {
            input: Input::String("hello".to_string()),
            output: None,
            map: None,
        });
        let result =
            InputExpression::from_special(&Special::Input, &params).unwrap();
        assert!(matches!(result, InputExpression::String(s) if s == "hello"));
    }

    #[test]
    fn special_input_returns_object_input_expression() {
        let mut obj = IndexMap::new();
        obj.insert("x".to_string(), Input::Integer(42));
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj),
            output: None,
            map: None,
        });
        let result =
            InputExpression::from_special(&Special::Input, &params).unwrap();
        match result {
            InputExpression::Object(map) => {
                assert!(map.contains_key("x"));
            }
            other => {
                panic!("expected InputExpression::Object, got {:?}", other)
            }
        }
    }

    // ── Special::Input failures ─────────────────────────────────────────

    #[test]
    fn special_input_fails_for_bool() {
        let params = Params::Owned(ParamsOwned {
            input: Input::String("hello".to_string()),
            output: None,
            map: None,
        });
        let result = bool::from_special(&Special::Input, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_input_fails_for_function_output() {
        let params = Params::Owned(ParamsOwned {
            input: Input::String("hello".to_string()),
            output: None,
            map: None,
        });
        let result = FunctionOutput::from_special(&Special::Input, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    // ── Special::Output ─────────────────────────────────────────────────

    #[test]
    fn special_output_returns_scalar() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::Function(FunctionOutput::Scalar(
                dec!(0.75),
            ))),
            map: None,
        });
        let result =
            FunctionOutput::from_special(&Special::Output, &params).unwrap();
        assert!(matches!(result, FunctionOutput::Scalar(d) if d == dec!(0.75)));
    }

    #[test]
    fn special_output_returns_vector() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::Function(FunctionOutput::Vector(
                vec![dec!(0.3), dec!(0.7)],
            ))),
            map: None,
        });
        let result =
            FunctionOutput::from_special(&Special::Output, &params).unwrap();
        match result {
            FunctionOutput::Vector(v) => {
                assert_eq!(v, vec![dec!(0.3), dec!(0.7)])
            }
            other => panic!("expected Vector, got {:?}", other),
        }
    }

    // ── Special::Output failures ────────────────────────────────────────

    #[test]
    fn special_output_fails_for_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::Function(FunctionOutput::Scalar(
                dec!(0.5),
            ))),
            map: None,
        });
        let result = Input::from_special(&Special::Output, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_output_fails_for_u64() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::Function(FunctionOutput::Scalar(
                dec!(0.5),
            ))),
            map: None,
        });
        let result = u64::from_special(&Special::Output, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    // ── Special::L1NormalizedFunctionOutput ──────────────────────────────

    #[test]
    fn special_l1_norm_normalizes_vector() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::Function(FunctionOutput::Vector(
                vec![dec!(2), dec!(3), dec!(5)],
            ))),
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::L1NormalizedFunctionOutput,
            &params,
        )
        .unwrap();
        match result {
            FunctionOutput::Vector(v) => {
                assert_eq!(v, vec![dec!(0.2), dec!(0.3), dec!(0.5)]);
            }
            other => panic!("expected Vector, got {:?}", other),
        }
    }

    #[test]
    fn special_l1_norm_normalizes_map_scalars() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::MapFunction(vec![
                FunctionOutput::Scalar(dec!(1)),
                FunctionOutput::Scalar(dec!(3)),
            ])),
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::L1NormalizedFunctionOutput,
            &params,
        )
        .unwrap();
        match result {
            FunctionOutput::Vector(v) => {
                assert_eq!(v, vec![dec!(0.25), dec!(0.75)]);
            }
            other => panic!("expected Vector, got {:?}", other),
        }
    }

    // ── Special::L1NormalizedFunctionOutput failures ─────────────────────

    #[test]
    fn special_l1_norm_fails_for_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::Function(FunctionOutput::Vector(
                vec![dec!(1)],
            ))),
            map: None,
        });
        let result =
            Input::from_special(&Special::L1NormalizedFunctionOutput, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_l1_norm_fails_for_string() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::Function(FunctionOutput::Vector(
                vec![dec!(1)],
            ))),
            map: None,
        });
        let result =
            String::from_special(&Special::L1NormalizedFunctionOutput, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    // ── Special::InputItemsOutputLength ──────────────────────────────────

    #[test]
    fn special_items_output_length_returns_count() {
        let mut obj = IndexMap::new();
        obj.insert(
            "items".to_string(),
            Input::Array(vec![
                Input::String("a".to_string()),
                Input::String("b".to_string()),
                Input::String("c".to_string()),
            ]),
        );
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj),
            output: None,
            map: None,
        });
        let result =
            u64::from_special(&Special::InputItemsOutputLength, &params)
                .unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn special_items_output_length_returns_zero_for_empty() {
        let mut obj = IndexMap::new();
        obj.insert("items".to_string(), Input::Array(vec![]));
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj),
            output: None,
            map: None,
        });
        let result =
            u64::from_special(&Special::InputItemsOutputLength, &params)
                .unwrap();
        assert_eq!(result, 0);
    }

    // ── Special::InputItemsOutputLength failures ────────────────────────

    #[test]
    fn special_items_output_length_fails_for_non_object_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::String("hello".to_string()),
            output: None,
            map: None,
        });
        let result =
            u64::from_special(&Special::InputItemsOutputLength, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_items_output_length_fails_for_missing_items() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Input::String("alice".to_string()));
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj),
            output: None,
            map: None,
        });
        let result =
            u64::from_special(&Special::InputItemsOutputLength, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    // ── Special::InputItemsOptionalContextSplit ─────────────────────────

    #[test]
    fn special_split_with_context() {
        let mut obj = IndexMap::new();
        obj.insert(
            "items".to_string(),
            Input::Array(vec![
                Input::String("x".to_string()),
                Input::String("y".to_string()),
            ]),
        );
        obj.insert("context".to_string(), Input::String("ctx".to_string()));
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj),
            output: None,
            map: None,
        });
        let result = Vec::<Input>::from_special(
            &Special::InputItemsOptionalContextSplit,
            &params,
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        // First element: items=["x"], context="ctx"
        match &result[0] {
            Input::Object(m) => {
                assert_eq!(
                    m.get("items").unwrap(),
                    &Input::Array(vec![Input::String("x".to_string())])
                );
                assert_eq!(
                    m.get("context").unwrap(),
                    &Input::String("ctx".to_string())
                );
            }
            other => panic!("expected Object, got {:?}", other),
        }
        // Second element: items=["y"], context="ctx"
        match &result[1] {
            Input::Object(m) => {
                assert_eq!(
                    m.get("items").unwrap(),
                    &Input::Array(vec![Input::String("y".to_string())])
                );
                assert_eq!(
                    m.get("context").unwrap(),
                    &Input::String("ctx".to_string())
                );
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }

    #[test]
    fn special_split_without_context() {
        let mut obj = IndexMap::new();
        obj.insert(
            "items".to_string(),
            Input::Array(vec![
                Input::Integer(1),
                Input::Integer(2),
                Input::Integer(3),
            ]),
        );
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj),
            output: None,
            map: None,
        });
        let result = Vec::<Input>::from_special(
            &Special::InputItemsOptionalContextSplit,
            &params,
        )
        .unwrap();
        assert_eq!(result.len(), 3);
        match &result[0] {
            Input::Object(m) => {
                assert_eq!(
                    m.get("items").unwrap(),
                    &Input::Array(vec![Input::Integer(1)])
                );
                assert!(m.get("context").is_none());
            }
            other => panic!("expected Object, got {:?}", other),
        }
        match &result[2] {
            Input::Object(m) => {
                assert_eq!(
                    m.get("items").unwrap(),
                    &Input::Array(vec![Input::Integer(3)])
                );
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }

    // ── Special::InputItemsOptionalContextSplit failures ─────────────────

    #[test]
    fn special_split_fails_for_non_object_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::String("hello".to_string()),
            output: None,
            map: None,
        });
        let result = Vec::<Input>::from_special(
            &Special::InputItemsOptionalContextSplit,
            &params,
        );
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_split_fails_for_missing_items() {
        let mut obj = IndexMap::new();
        obj.insert("name".to_string(), Input::String("alice".to_string()));
        let params = Params::Owned(ParamsOwned {
            input: Input::Object(obj),
            output: None,
            map: None,
        });
        let result = Vec::<Input>::from_special(
            &Special::InputItemsOptionalContextSplit,
            &params,
        );
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    // ── Special::InputItemsOptionalContextMerge ─────────────────────────

    #[test]
    fn special_merge_with_context() {
        let sub1 = {
            let mut m = IndexMap::new();
            m.insert(
                "items".to_string(),
                Input::Array(vec![Input::String("a".to_string())]),
            );
            m.insert("context".to_string(), Input::String("ctx".to_string()));
            Input::Object(m)
        };
        let sub2 = {
            let mut m = IndexMap::new();
            m.insert(
                "items".to_string(),
                Input::Array(vec![Input::String("b".to_string())]),
            );
            m.insert("context".to_string(), Input::String("ctx".to_string()));
            Input::Object(m)
        };
        let params = Params::Owned(ParamsOwned {
            input: Input::Array(vec![sub1, sub2]),
            output: None,
            map: None,
        });
        let result = Input::from_special(
            &Special::InputItemsOptionalContextMerge,
            &params,
        )
        .unwrap();
        match result {
            Input::Object(m) => {
                assert_eq!(
                    m.get("items").unwrap(),
                    &Input::Array(vec![
                        Input::String("a".to_string()),
                        Input::String("b".to_string()),
                    ])
                );
                assert_eq!(
                    m.get("context").unwrap(),
                    &Input::String("ctx".to_string())
                );
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }

    #[test]
    fn special_merge_without_context() {
        let sub1 = {
            let mut m = IndexMap::new();
            m.insert(
                "items".to_string(),
                Input::Array(vec![Input::Integer(10)]),
            );
            Input::Object(m)
        };
        let sub2 = {
            let mut m = IndexMap::new();
            m.insert(
                "items".to_string(),
                Input::Array(vec![Input::Integer(20)]),
            );
            Input::Object(m)
        };
        let sub3 = {
            let mut m = IndexMap::new();
            m.insert(
                "items".to_string(),
                Input::Array(vec![Input::Integer(30)]),
            );
            Input::Object(m)
        };
        let params = Params::Owned(ParamsOwned {
            input: Input::Array(vec![sub1, sub2, sub3]),
            output: None,
            map: None,
        });
        let result = Input::from_special(
            &Special::InputItemsOptionalContextMerge,
            &params,
        )
        .unwrap();
        match result {
            Input::Object(m) => {
                assert_eq!(
                    m.get("items").unwrap(),
                    &Input::Array(vec![
                        Input::Integer(10),
                        Input::Integer(20),
                        Input::Integer(30)
                    ])
                );
                assert!(m.get("context").is_none());
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }

    // ── Special::InputItemsOptionalContextMerge failures ─────────────────

    #[test]
    fn special_merge_fails_for_non_array_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::String("hello".to_string()),
            output: None,
            map: None,
        });
        let result = Input::from_special(
            &Special::InputItemsOptionalContextMerge,
            &params,
        );
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_merge_fails_for_non_object_elements() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Array(vec![Input::Integer(1), Input::Integer(2)]),
            output: None,
            map: None,
        });
        let result = Input::from_special(
            &Special::InputItemsOptionalContextMerge,
            &params,
        );
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    // ── Special::VectorCompletionScores ──────────────────────────────────

    #[test]
    fn special_vc_scores_returns_scores() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::VectorCompletion(
                VectorCompletionOutput {
                    votes: vec![make_vote()],
                    scores: vec![dec!(0.6), dec!(0.4)],
                    weights: vec![dec!(1), dec!(0)],
                },
            )),
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::VectorCompletionScores,
            &params,
        )
        .unwrap();
        match result {
            FunctionOutput::Vector(v) => {
                assert_eq!(v, vec![dec!(0.6), dec!(0.4)])
            }
            other => panic!("expected Vector, got {:?}", other),
        }
    }

    #[test]
    fn special_vc_scores_returns_three_scores() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::VectorCompletion(
                VectorCompletionOutput {
                    votes: vec![make_vote()],
                    scores: vec![dec!(0.2), dec!(0.3), dec!(0.5)],
                    weights: vec![dec!(1), dec!(0), dec!(0)],
                },
            )),
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::VectorCompletionScores,
            &params,
        )
        .unwrap();
        match result {
            FunctionOutput::Vector(v) => {
                assert_eq!(v, vec![dec!(0.2), dec!(0.3), dec!(0.5)])
            }
            other => panic!("expected Vector, got {:?}", other),
        }
    }

    // ── Special::VectorCompletionScores failures ────────────────────────

    #[test]
    fn special_vc_scores_fails_for_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::VectorCompletion(
                VectorCompletionOutput {
                    votes: vec![make_vote()],
                    scores: vec![dec!(0.5), dec!(0.5)],
                    weights: vec![dec!(1), dec!(0)],
                },
            )),
            map: None,
        });
        let result =
            Input::from_special(&Special::VectorCompletionScores, &params);
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_vc_scores_fails_for_no_output() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: None,
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::VectorCompletionScores,
            &params,
        );
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    // ── Special::VectorCompletionScoresWeightedSum ───────────────────────

    #[test]
    fn special_vc_weighted_sum_two_scores() {
        // scores=[0.6, 0.4], weights=[0/1, 1/1]=[0, 1]
        // weighted_sum = 0.6*0 + 0.4*1 = 0.4
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::VectorCompletion(
                VectorCompletionOutput {
                    votes: vec![make_vote()],
                    scores: vec![dec!(0.6), dec!(0.4)],
                    weights: vec![dec!(1), dec!(0)],
                },
            )),
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::VectorCompletionScoresWeightedSum,
            &params,
        )
        .unwrap();
        assert!(matches!(result, FunctionOutput::Scalar(d) if d == dec!(0.4)));
    }

    #[test]
    fn special_vc_weighted_sum_three_scores() {
        // scores=[0.2, 0.3, 0.5], weights=[0/2, 1/2, 2/2]=[0, 0.5, 1]
        // weighted_sum = 0.2*0 + 0.3*0.5 + 0.5*1 = 0 + 0.15 + 0.5 = 0.65
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::VectorCompletion(
                VectorCompletionOutput {
                    votes: vec![make_vote()],
                    scores: vec![dec!(0.2), dec!(0.3), dec!(0.5)],
                    weights: vec![dec!(1), dec!(0), dec!(0)],
                },
            )),
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::VectorCompletionScoresWeightedSum,
            &params,
        )
        .unwrap();
        assert!(matches!(result, FunctionOutput::Scalar(d) if d == dec!(0.65)));
    }

    // ── Special::VectorCompletionScoresWeightedSum failures ──────────────

    #[test]
    fn special_vc_weighted_sum_fails_for_input() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: Some(TaskOutputOwned::VectorCompletion(
                VectorCompletionOutput {
                    votes: vec![make_vote()],
                    scores: vec![dec!(0.5), dec!(0.5)],
                    weights: vec![dec!(1), dec!(0)],
                },
            )),
            map: None,
        });
        let result = Input::from_special(
            &Special::VectorCompletionScoresWeightedSum,
            &params,
        );
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }

    #[test]
    fn special_vc_weighted_sum_fails_for_no_output() {
        let params = Params::Owned(ParamsOwned {
            input: Input::Boolean(true),
            output: None,
            map: None,
        });
        let result = FunctionOutput::from_special(
            &Special::VectorCompletionScoresWeightedSum,
            &params,
        );
        assert!(matches!(result, Err(ExpressionError::UnsupportedSpecial)));
    }
}
