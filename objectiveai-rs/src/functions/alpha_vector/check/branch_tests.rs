//! Tests for check_alpha_branch_vector_function.

#![cfg(test)]

use crate::functions::alpha_vector::expression::{
    VectorFunctionInputExpression, VectorFunctionInputSchema,
};
use crate::functions::alpha_vector::{
    BranchTaskExpression, PlaceholderScalarFunctionTaskExpression,
    PlaceholderVectorFunctionTaskExpression, RemoteFunction,
    ScalarFunctionTaskExpression, VectorFunctionTaskExpression,
};
use crate::functions::expression::{
    ArrayInputSchema, BooleanInputSchema, Expression, InputSchema,
    IntegerInputSchema, ObjectInputSchema, StringInputSchema,
};
use crate::functions::alpha_vector::check::check_alpha_branch_vector_function;
use crate::functions::Remote;
use crate::util::index_map;

fn test(f: &RemoteFunction) {
    check_alpha_branch_vector_function(f, None).unwrap();
}

fn test_err(f: &RemoteFunction, expected: &str) {
    let err = check_alpha_branch_vector_function(f, None).unwrap_err();
    assert!(
        err.contains(expected),
        "expected '{expected}' in error, got: {err}"
    );
}

// --- Structural checks ---

#[test]
fn wrong_type_leaf() {
    let f = RemoteFunction::Leaf {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![],
    };
    test_err(&f, "AW01");
}

#[test]
fn description_empty() {
    let f = RemoteFunction::Branch {
        description: "  ".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::VectorFunction(
            VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            },
        )],
    };
    test_err(&f, "QD01");
}

#[test]
fn description_too_long() {
    let f = RemoteFunction::Branch {
        description: "a".repeat(351),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::VectorFunction(
            VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            },
        )],
    };
    test_err(&f, "QD02");
}

#[test]
fn no_tasks() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![],
    };
    test_err(&f, "AW02");
}

// --- Composition rules ---

#[test]
fn single_scalar_task_rejected() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::ScalarFunction(
            ScalarFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: Expression::Starlark("input".to_string()),
            },
        )],
    };
    test_err(&f, "AW08");
}

#[test]
fn single_placeholder_scalar_task_rejected() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::PlaceholderScalarFunction(
            PlaceholderScalarFunctionTaskExpression {
                params: crate::functions::inventions::Params {
                    depth: 1,
                    min_branch_width: 1,
                    max_branch_width: 3,
                    min_leaf_width: 1,
                    max_leaf_width: 3,
                    spec: "Score each item".to_string(),
                    name: "test".to_string(),
                },
                input_schema: ObjectInputSchema {
                    description: None,
                    properties: index_map! {
                        "text" => InputSchema::String(StringInputSchema {
                            description: None,
                            r#enum: None,
                        })
                    },
                    required: Some(vec!["text".to_string()]),
                },
                skip: None,
                input: Expression::Starlark("input".to_string()),
            },
        )],
    };
    test_err(&f, "AW08");
}

#[test]
fn over_50_percent_scalar_tasks() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::ScalarFunction(ScalarFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: Expression::Starlark("input".to_string()),
            }),
            BranchTaskExpression::ScalarFunction(ScalarFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test2".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: Expression::Starlark("input".to_string()),
            }),
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
        ],
    };
    test_err(&f, "AW09");
}

// --- Success cases ---

#[test]
fn valid_single_vector_function() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::VectorFunction(
            VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            },
        )],
    };
    test(&f);
}

#[test]
fn valid_single_placeholder_vector() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::PlaceholderVectorFunction(
            PlaceholderVectorFunctionTaskExpression {
                params: crate::functions::inventions::Params {
                    depth: 1,
                    min_branch_width: 1,
                    max_branch_width: 3,
                    min_leaf_width: 1,
                    max_leaf_width: 3,
                    spec: "Rank items".to_string(),
                    name: "test".to_string(),
                },
                input_schema: VectorFunctionInputSchema {
                    context: None,
                    items: InputSchema::Array(ArrayInputSchema {
                        description: None,
                        min_items: Some(2),
                        max_items: Some(10),
                        items: Box::new(InputSchema::String(StringInputSchema {
                            description: None,
                            r#enum: None,
                        })),
                    }),
                },
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            },
        )],
    };
    test(&f);
}

#[test]
fn valid_all_vector_tasks() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test2".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
        ],
    };
    test(&f);
}

#[test]
fn valid_mixed_placeholder_vector_tasks() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
            BranchTaskExpression::PlaceholderVectorFunction(
                PlaceholderVectorFunctionTaskExpression {
                    params: crate::functions::inventions::Params {
                        depth: 1,
                        min_branch_width: 1,
                        max_branch_width: 3,
                        min_leaf_width: 1,
                        max_leaf_width: 3,
                        spec: "Rank items differently".to_string(),
                        name: "test".to_string(),
                    },
                    input_schema: VectorFunctionInputSchema {
                        context: None,
                        items: InputSchema::Array(ArrayInputSchema {
                            description: None,
                            min_items: Some(2),
                            max_items: Some(10),
                            items: Box::new(InputSchema::String(StringInputSchema {
                                description: None,
                                r#enum: None,
                            })),
                        }),
                    },
                    skip: None,
                    input: VectorFunctionInputExpression {
                        context: None,
                        items: Expression::Starlark(
                            "[x + ' alt' for x in input['items']]".to_string(),
                        ),
                    },
                },
            ),
        ],
    };
    test(&f);
}

// --- Diversity checks ---

#[test]
fn input_diversity_fail_fixed_input() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test2".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("['A', 'B']".to_string()),
                },
            }),
        ],
    };
    test_err(&f, "AW18");
}

#[test]
fn input_diversity_pass_all_derived() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test2".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
        ],
    };
    test(&f);
}

#[test]
fn input_diversity_pass_placeholder_vector_tasks() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::PlaceholderVectorFunction(
                PlaceholderVectorFunctionTaskExpression {
                    params: crate::functions::inventions::Params {
                        depth: 1,
                        min_branch_width: 1,
                        max_branch_width: 3,
                        min_leaf_width: 1,
                        max_leaf_width: 3,
                        spec: "Rank items by quality".to_string(),
                        name: "test".to_string(),
                    },
                    input_schema: VectorFunctionInputSchema {
                        context: None,
                        items: InputSchema::Array(ArrayInputSchema {
                            description: None,
                            min_items: Some(2),
                            max_items: Some(10),
                            items: Box::new(InputSchema::String(StringInputSchema {
                                description: None,
                                r#enum: None,
                            })),
                        }),
                    },
                    skip: None,
                    input: VectorFunctionInputExpression {
                        context: None,
                        items: Expression::Starlark("input['items']".to_string()),
                    },
                },
            ),
            BranchTaskExpression::PlaceholderVectorFunction(
                PlaceholderVectorFunctionTaskExpression {
                    params: crate::functions::inventions::Params {
                        depth: 1,
                        min_branch_width: 1,
                        max_branch_width: 3,
                        min_leaf_width: 1,
                        max_leaf_width: 3,
                        spec: "Rank items by relevance".to_string(),
                        name: "test".to_string(),
                    },
                    input_schema: VectorFunctionInputSchema {
                        context: None,
                        items: InputSchema::Array(ArrayInputSchema {
                            description: None,
                            min_items: Some(2),
                            max_items: Some(10),
                            items: Box::new(InputSchema::String(StringInputSchema {
                                description: None,
                                r#enum: None,
                            })),
                        }),
                    },
                    skip: None,
                    input: VectorFunctionInputExpression {
                        context: None,
                        items: Expression::Starlark(
                            "[x + ' alt' for x in input['items']]".to_string(),
                        ),
                    },
                },
            ),
        ],
    };
    test(&f);
}

// --- All tasks skipped ---

#[test]
fn all_tasks_skipped() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: Some(Expression::Starlark("True".to_string())),
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test2".to_string(),
                commit: "abc123".to_string(),
                skip: Some(Expression::Starlark("True".to_string())),
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
        ],
    };
    test_err(&f, "CV42");
}

// --- Input schema permutation checks ---

#[test]
fn rejects_single_permutation_string_enum() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(2),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: Some(vec!["only".to_string()]),
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::VectorFunction(
            VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            },
        )],
    };
    test_err(&f, "QI01");
}

#[test]
fn rejects_single_permutation_integer() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(2),
                items: Box::new(InputSchema::Integer(IntegerInputSchema {
                    description: None,
                    minimum: Some(0),
                    maximum: Some(0),
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::VectorFunction(
            VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            },
        )],
    };
    test_err(&f, "QI01");
}

// --- Placeholder field validation ---
//
// AW21 (placeholder scalar) triggers check_scalar_fields on the transpiled
// placeholder's input_schema. The input_schema must have < 2 permutations
// (QI01). But we also need the generate loop to succeed, which means the
// compiled input must match the placeholder's schema, and the scalar output
// must be valid. Since scalar tasks with map: None produce
// TaskOutputOwned::Scalar in a vector parent (CV16), AW21 for scalar
// placeholders is unreachable with children=None. We test it indirectly
// by showing the composition check (AW08) catches single scalar placeholders.
//
// AW22 (placeholder vector) triggers check_vector_fields on the transpiled
// placeholder's input_schema. We use min_items: 1 so that VF03
// (output_length < 2) fires during the placeholder's own field validation.

#[test]
fn placeholder_vector_field_validation() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::String(StringInputSchema {
                    description: None,
                    r#enum: None,
                })),
            }),
        },
        tasks: vec![BranchTaskExpression::PlaceholderVectorFunction(
            PlaceholderVectorFunctionTaskExpression {
                params: crate::functions::inventions::Params {
                    depth: 1,
                    min_branch_width: 1,
                    max_branch_width: 3,
                    min_leaf_width: 1,
                    max_leaf_width: 3,
                    spec: "Rank items".to_string(),
                    name: "test".to_string(),
                },
                input_schema: VectorFunctionInputSchema {
                    context: None,
                    items: InputSchema::Array(ArrayInputSchema {
                        description: None,
                        min_items: Some(1),
                        max_items: Some(10),
                        items: Box::new(InputSchema::String(StringInputSchema {
                            description: None,
                            r#enum: None,
                        })),
                    }),
                },
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            },
        )],
    };
    test_err(&f, "AW22");
}

// --- Skip expression tests ---
//
// Note: skip tests with context are not included because
// InputItemsOptionalContextMerge produces keys in {items, context} order
// while the transpiled schema produces {context, items} order, causing VF12
// key order mismatch. Skip tests without context work fine.

#[test]
fn valid_with_skip_on_boolean() {
    let f = RemoteFunction::Branch {
        description: "test".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::Object(ObjectInputSchema {
                    description: None,
                    properties: index_map! {
                        "text" => InputSchema::String(StringInputSchema {
                            description: None,
                            r#enum: None,
                        }),
                        "skip_last" => InputSchema::Boolean(BooleanInputSchema {
                            description: None,
                        })
                    },
                    required: Some(vec!["text".to_string(), "skip_last".to_string()]),
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test2".to_string(),
                commit: "abc123".to_string(),
                skip: Some(Expression::Starlark(
                    "input['items'][0]['skip_last']".to_string(),
                )),
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
        ],
    };
    test(&f);
}

#[test]
fn valid_with_skip_on_mode_enum() {
    let f = RemoteFunction::Branch {
        description: "Rank with optional deep analysis".to_string(),
        input_schema: VectorFunctionInputSchema {
            context: None,
            items: InputSchema::Array(ArrayInputSchema {
                description: None,
                min_items: Some(2),
                max_items: Some(10),
                items: Box::new(InputSchema::Object(ObjectInputSchema {
                    description: None,
                    properties: index_map! {
                        "text" => InputSchema::String(StringInputSchema {
                            description: None,
                            r#enum: None,
                        }),
                        "mode" => InputSchema::String(StringInputSchema {
                            description: None,
                            r#enum: Some(vec!["quick".to_string(), "thorough".to_string()]),
                        })
                    },
                    required: Some(vec!["text".to_string(), "mode".to_string()]),
                })),
            }),
        },
        tasks: vec![
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test".to_string(),
                commit: "abc123".to_string(),
                skip: None,
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
            BranchTaskExpression::VectorFunction(VectorFunctionTaskExpression {
                remote: Remote::Github,
                owner: "test".to_string(),
                repository: "test2".to_string(),
                commit: "abc123".to_string(),
                skip: Some(Expression::Starlark(
                    "input['items'][0]['mode'] == 'quick'".to_string(),
                )),
                input: VectorFunctionInputExpression {
                    context: None,
                    items: Expression::Starlark("input['items']".to_string()),
                },
            }),
        ],
    };
    test(&f);
}
