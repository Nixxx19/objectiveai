use crate::tests::stream_push::stream_push_test;
use super::*;

stream_push_test!(
    single_chunk_unchanged,
    vec![LaboratoryExecutionChunk {
        id: "lec-1".into(),
        builders: vec![],
        evaluations: vec![],
        error: None,
        created: 100,
        object: Object::LaboratoryExecutionChunk,
        usage: None,
    }],
    LaboratoryExecutionChunk {
        id: "lec-1".into(),
        builders: vec![],
        evaluations: vec![],
        error: None,
        created: 100,
        object: Object::LaboratoryExecutionChunk,
        usage: None,
    }
);

stream_push_test!(
    error_replaced_by_later_chunk,
    vec![
        LaboratoryExecutionChunk {
            id: "lec-2".into(),
            builders: vec![],
            evaluations: vec![],
            error: Some(crate::error::ResponseError {
                code: 500,
                message: serde_json::json!("first"),
            }),
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: None,
        },
        LaboratoryExecutionChunk {
            id: "lec-2".into(),
            builders: vec![],
            evaluations: vec![],
            error: Some(crate::error::ResponseError {
                code: 502,
                message: serde_json::json!("second"),
            }),
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: None,
        },
    ],
    LaboratoryExecutionChunk {
        id: "lec-2".into(),
        builders: vec![],
        evaluations: vec![],
        error: Some(crate::error::ResponseError {
            code: 502,
            message: serde_json::json!("second"),
        }),
        created: 100,
        object: Object::LaboratoryExecutionChunk,
        usage: None,
    }
);

stream_push_test!(
    usage_set_from_later_chunk,
    vec![
        LaboratoryExecutionChunk {
            id: "lec-3".into(),
            builders: vec![],
            evaluations: vec![],
            error: None,
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: None,
        },
        LaboratoryExecutionChunk {
            id: "lec-3".into(),
            builders: vec![],
            evaluations: vec![],
            error: None,
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: Some(crate::agent::completions::response::Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                completion_tokens_details: None,
                prompt_tokens_details: None,
                cost: rust_decimal::Decimal::ZERO,
                cost_details: None,
                total_cost: rust_decimal::Decimal::ZERO,
            }),
        },
    ],
    LaboratoryExecutionChunk {
        id: "lec-3".into(),
        builders: vec![],
        evaluations: vec![],
        error: None,
        created: 100,
        object: Object::LaboratoryExecutionChunk,
        usage: Some(crate::agent::completions::response::Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
            completion_tokens_details: None,
            prompt_tokens_details: None,
            cost: rust_decimal::Decimal::ZERO,
            cost_details: None,
            total_cost: rust_decimal::Decimal::ZERO,
        }),
    }
);

stream_push_test!(
    builders_merged_by_index,
    vec![
        LaboratoryExecutionChunk {
            id: "lec-4".into(),
            builders: vec![BuilderChunk {
                index: 0,
                agent_index: 0,
                inner: Default::default(),
            }],
            evaluations: vec![],
            error: None,
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: None,
        },
        LaboratoryExecutionChunk {
            id: "lec-4".into(),
            builders: vec![
                BuilderChunk {
                    index: 0,
                    agent_index: 0,
                    inner: Default::default(),
                },
                BuilderChunk {
                    index: 1,
                    agent_index: 1,
                    inner: Default::default(),
                },
            ],
            evaluations: vec![],
            error: None,
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: None,
        },
    ],
    LaboratoryExecutionChunk {
        id: "lec-4".into(),
        builders: vec![
            BuilderChunk {
                index: 0,
                agent_index: 0,
                inner: Default::default(),
            },
            BuilderChunk {
                index: 1,
                agent_index: 1,
                inner: Default::default(),
            },
        ],
        evaluations: vec![],
        error: None,
        created: 100,
        object: Object::LaboratoryExecutionChunk,
        usage: None,
    }
);

stream_push_test!(
    evaluations_merged_by_index,
    vec![
        LaboratoryExecutionChunk {
            id: "lec-5".into(),
            builders: vec![],
            evaluations: vec![EvaluationChunk {
                index: 0,
                agent_index: 0,
                inner: Default::default(),
                output: None,
            }],
            error: None,
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: None,
        },
        LaboratoryExecutionChunk {
            id: "lec-5".into(),
            builders: vec![],
            evaluations: vec![EvaluationChunk {
                index: 0,
                agent_index: 0,
                inner: Default::default(),
                output: Some(crate::functions::expression::InputValue::Integer(42)),
            }],
            error: None,
            created: 100,
            object: Object::LaboratoryExecutionChunk,
            usage: None,
        },
    ],
    LaboratoryExecutionChunk {
        id: "lec-5".into(),
        builders: vec![],
        evaluations: vec![EvaluationChunk {
            index: 0,
            agent_index: 0,
            inner: Default::default(),
            output: Some(crate::functions::expression::InputValue::Integer(42)),
        }],
        error: None,
        created: 100,
        object: Object::LaboratoryExecutionChunk,
        usage: None,
    }
);
