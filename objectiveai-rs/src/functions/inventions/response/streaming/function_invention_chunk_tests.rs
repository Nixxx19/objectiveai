use crate::tests::stream_push::stream_push_test;
use super::*;

stream_push_test!(
    single_chunk_unchanged,
    vec![FunctionInventionChunk {
        id: "fic-1".into(),
        completions: vec![],
        state: None,
        path: None,
        function: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionChunk,
        usage: None,
        error: None,
    }],
    FunctionInventionChunk {
        id: "fic-1".into(),
        completions: vec![],
        state: None,
        path: None,
        function: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionChunk,
        usage: None,
        error: None,
    }
);

stream_push_test!(
    completions_merged_by_index,
    vec![
        FunctionInventionChunk {
            id: "fic-2".into(),
            completions: vec![AgentCompletionChunk {
                index: 0,
                inner: crate::agent::completions::response::streaming::AgentCompletionChunk {
                    id: "acc-1".into(),
                    created: 0,
                    messages: vec![],
                    object: crate::agent::completions::response::streaming::Object::AgentCompletionChunk,
                    usage: None,
                    upstream: crate::agent::Upstream::Openrouter,
                    error: None,
                    continuation: None,
                },
            }],
            state: None,
            path: None,
            function: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionChunk,
            usage: None,
            error: None,
        },
        FunctionInventionChunk {
            id: "fic-2".into(),
            completions: vec![AgentCompletionChunk {
                index: 1,
                inner: crate::agent::completions::response::streaming::AgentCompletionChunk {
                    id: "acc-2".into(),
                    created: 0,
                    messages: vec![],
                    object: crate::agent::completions::response::streaming::Object::AgentCompletionChunk,
                    usage: None,
                    upstream: crate::agent::Upstream::Openrouter,
                    error: None,
                    continuation: None,
                },
            }],
            state: None,
            path: None,
            function: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionChunk,
            usage: None,
            error: None,
        },
    ],
    FunctionInventionChunk {
        id: "fic-2".into(),
        completions: vec![
            AgentCompletionChunk {
                index: 0,
                inner: crate::agent::completions::response::streaming::AgentCompletionChunk {
                    id: "acc-1".into(),
                    created: 0,
                    messages: vec![],
                    object: crate::agent::completions::response::streaming::Object::AgentCompletionChunk,
                    usage: None,
                    upstream: crate::agent::Upstream::Openrouter,
                    error: None,
                    continuation: None,
                },
            },
            AgentCompletionChunk {
                index: 1,
                inner: crate::agent::completions::response::streaming::AgentCompletionChunk {
                    id: "acc-2".into(),
                    created: 0,
                    messages: vec![],
                    object: crate::agent::completions::response::streaming::Object::AgentCompletionChunk,
                    usage: None,
                    upstream: crate::agent::Upstream::Openrouter,
                    error: None,
                    continuation: None,
                },
            },
        ],
        state: None,
        path: None,
        function: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionChunk,
        usage: None,
        error: None,
    }
);

stream_push_test!(
    usage_set_from_later_chunk,
    vec![
        FunctionInventionChunk {
            id: "fic-3".into(),
            completions: vec![],
            state: None,
            path: None,
            function: None,
            created: 100,
            object: Object::AlphaVectorFunctionInventionChunk,
            usage: None,
            error: None,
        },
        FunctionInventionChunk {
            id: "fic-3".into(),
            completions: vec![],
            state: None,
            path: None,
            function: None,
            created: 100,
            object: Object::AlphaVectorFunctionInventionChunk,
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
            error: None,
        },
    ],
    FunctionInventionChunk {
        id: "fic-3".into(),
        completions: vec![],
        state: None,
        path: None,
        function: None,
        created: 100,
        object: Object::AlphaVectorFunctionInventionChunk,
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
        error: None,
    }
);

stream_push_test!(
    error_replaced_by_later_chunk,
    vec![
        FunctionInventionChunk {
            id: "fic-4".into(),
            completions: vec![],
            state: None,
            path: None,
            function: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionChunk,
            usage: None,
            error: Some(crate::error::ResponseError {
                code: 500,
                message: serde_json::json!("first"),
            }),
        },
        FunctionInventionChunk {
            id: "fic-4".into(),
            completions: vec![],
            state: None,
            path: None,
            function: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionChunk,
            usage: None,
            error: Some(crate::error::ResponseError {
                code: 502,
                message: serde_json::json!("second"),
            }),
        },
    ],
    FunctionInventionChunk {
        id: "fic-4".into(),
        completions: vec![],
        state: None,
        path: None,
        function: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionChunk,
        usage: None,
        error: Some(crate::error::ResponseError {
            code: 502,
            message: serde_json::json!("second"),
        }),
    }
);

stream_push_test!(
    path_replaced_by_later_chunk,
    vec![
        FunctionInventionChunk {
            id: "fic-5".into(),
            completions: vec![],
            state: None,
            path: Some(crate::RemotePath::Github {
                owner: "owner-a".into(),
                repository: "repo-a".into(),
                commit: "aaa111".into(),
            }),
            function: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionChunk,
            usage: None,
            error: None,
        },
        FunctionInventionChunk {
            id: "fic-5".into(),
            completions: vec![],
            state: None,
            path: Some(crate::RemotePath::Github {
                owner: "owner-b".into(),
                repository: "repo-b".into(),
                commit: "abc123".into(),
            }),
            function: None,
            created: 100,
            object: Object::AlphaScalarFunctionInventionChunk,
            usage: None,
            error: None,
        },
    ],
    FunctionInventionChunk {
        id: "fic-5".into(),
        completions: vec![],
        state: None,
        path: Some(crate::RemotePath::Github {
            owner: "owner-b".into(),
            repository: "repo-b".into(),
            commit: "abc123".into(),
        }),
        function: None,
        created: 100,
        object: Object::AlphaScalarFunctionInventionChunk,
        usage: None,
        error: None,
    }
);
