use crate::tests::stream_push::stream_push_test;
use super::*;

stream_push_test!(
    single_chunk_unchanged,
    vec![FunctionInventionRecursiveChunk {
        id: "firc-1".into(),
        inventions: vec![],
        inventions_errors: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionRecursiveChunk,
        usage: None,
    }],
    FunctionInventionRecursiveChunk {
        id: "firc-1".into(),
        inventions: vec![],
        inventions_errors: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionRecursiveChunk,
        usage: None,
    }
);

stream_push_test!(
    inventions_merged_by_index,
    vec![
        FunctionInventionRecursiveChunk {
            id: "firc-2".into(),
            inventions: vec![FunctionInventionChunk {
                index: 0,
                inner: crate::functions::inventions::response::streaming::FunctionInventionChunk {
                    id: "fi-1".into(),
                    completions: vec![],
                    state: None,
                    path: None,
                    function: None,
                    created: 100,
                    object: crate::functions::inventions::response::streaming::Object::AlphaScalarFunctionInventionChunk,
                    usage: None,
                    error: None,
                },
            }],
            inventions_errors: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionRecursiveChunk,
            usage: None,
        },
        FunctionInventionRecursiveChunk {
            id: "firc-2".into(),
            inventions: vec![FunctionInventionChunk {
                index: 1,
                inner: crate::functions::inventions::response::streaming::FunctionInventionChunk {
                    id: "fi-2".into(),
                    completions: vec![],
                    state: None,
                    path: None,
                    function: None,
                    created: 100,
                    object: crate::functions::inventions::response::streaming::Object::AlphaScalarFunctionInventionChunk,
                    usage: None,
                    error: None,
                },
            }],
            inventions_errors: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionRecursiveChunk,
            usage: None,
        },
    ],
    FunctionInventionRecursiveChunk {
        id: "firc-2".into(),
        inventions: vec![
            FunctionInventionChunk {
                index: 0,
                inner: crate::functions::inventions::response::streaming::FunctionInventionChunk {
                    id: "fi-1".into(),
                    completions: vec![],
                    state: None,
                    path: None,
                    function: None,
                    created: 100,
                    object: crate::functions::inventions::response::streaming::Object::AlphaScalarFunctionInventionChunk,
                    usage: None,
                    error: None,
                },
            },
            FunctionInventionChunk {
                index: 1,
                inner: crate::functions::inventions::response::streaming::FunctionInventionChunk {
                    id: "fi-2".into(),
                    completions: vec![],
                    state: None,
                    path: None,
                    function: None,
                    created: 100,
                    object: crate::functions::inventions::response::streaming::Object::AlphaScalarFunctionInventionChunk,
                    usage: None,
                    error: None,
                },
            },
        ],
        inventions_errors: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionRecursiveChunk,
        usage: None,
    }
);

stream_push_test!(
    inventions_errors_set,
    vec![
        FunctionInventionRecursiveChunk {
            id: "firc-3".into(),
            inventions: vec![],
            inventions_errors: None,
            created: 100,
            object: Object::AlphaVectorFunctionInventionRecursiveChunk,
            usage: None,
        },
        FunctionInventionRecursiveChunk {
            id: "firc-3".into(),
            inventions: vec![],
            inventions_errors: Some(true),
            created: 100,
            object: Object::AlphaVectorFunctionInventionRecursiveChunk,
            usage: None,
        },
    ],
    FunctionInventionRecursiveChunk {
        id: "firc-3".into(),
        inventions: vec![],
        inventions_errors: Some(true),
        created: 100,
        object: Object::AlphaVectorFunctionInventionRecursiveChunk,
        usage: None,
    }
);

stream_push_test!(
    usage_set_from_later_chunk,
    vec![
        FunctionInventionRecursiveChunk {
            id: "firc-4".into(),
            inventions: vec![],
            inventions_errors: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionRecursiveChunk,
            usage: None,
        },
        FunctionInventionRecursiveChunk {
            id: "firc-4".into(),
            inventions: vec![],
            inventions_errors: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionRecursiveChunk,
            usage: Some(crate::agent::completions::response::Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                completion_tokens_details: None,
                prompt_tokens_details: None,
                cost: rust_decimal::Decimal::new(1, 3),
                cost_details: None,
                total_cost: rust_decimal::Decimal::new(1, 3),
            }),
        },
    ],
    FunctionInventionRecursiveChunk {
        id: "firc-4".into(),
        inventions: vec![],
        inventions_errors: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionRecursiveChunk,
        usage: Some(crate::agent::completions::response::Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            cost: rust_decimal::Decimal::new(1, 3),
            cost_details: None,
            total_cost: rust_decimal::Decimal::new(1, 3),
        }),
    }
);

stream_push_test!(
    usage_additive_across_chunks,
    vec![
        FunctionInventionRecursiveChunk {
            id: "firc-5".into(),
            inventions: vec![],
            inventions_errors: None,
            created: 200,
            object: Object::AlphaScalarFunctionInventionRecursiveChunk,
            usage: Some(crate::agent::completions::response::Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                completion_tokens_details: None,
                prompt_tokens_details: None,
                cost: rust_decimal::Decimal::new(1, 3),
                cost_details: None,
                total_cost: rust_decimal::Decimal::new(1, 3),
            }),
        },
        FunctionInventionRecursiveChunk {
            id: "firc-5".into(),
            inventions: vec![],
            inventions_errors: None,
            created: 200,
            object: Object::AlphaScalarFunctionInventionRecursiveChunk,
            usage: Some(crate::agent::completions::response::Usage {
                prompt_tokens: 20,
                completion_tokens: 10,
                total_tokens: 30,
                completion_tokens_details: None,
                prompt_tokens_details: None,
                cost: rust_decimal::Decimal::new(2, 3),
                cost_details: None,
                total_cost: rust_decimal::Decimal::new(2, 3),
            }),
        },
    ],
    FunctionInventionRecursiveChunk {
        id: "firc-5".into(),
        inventions: vec![],
        inventions_errors: None,
        created: 200,
        object: Object::AlphaScalarFunctionInventionRecursiveChunk,
        usage: Some(crate::agent::completions::response::Usage {
            prompt_tokens: 30,
            completion_tokens: 15,
            total_tokens: 45,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            cost: rust_decimal::Decimal::new(3, 3),
            cost_details: None,
            total_cost: rust_decimal::Decimal::new(3, 3),
        }),
    }
);
