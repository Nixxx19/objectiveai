use crate::functions;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaScalarLeafState {
    #[serde(flatten)]
    pub params: super::Params,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub essay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema:
        Option<functions::alpha_scalar::expression::ScalarFunctionInputSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub essay_tasks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<functions::alpha_scalar::LeafTaskExpression>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
}

impl AlphaScalarLeafState {
    pub fn read_spec_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "ReadSpec",
            "Read Spec",
            crate::upstream::ToolArgsType::None,
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    Ok(state.params.spec.clone())
                }
            },
        )
    }

    pub fn read_essay_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "ReadEssay",
            "Read Essay",
            crate::upstream::ToolArgsType::None,
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match &state.essay {
                        Some(essay) => Ok(essay.clone()),
                        None => Err("Essay has not been written".to_string()),
                    }
                }
            },
        )
    }

    pub fn write_essay_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "WriteEssay",
            "Write Essay",
            crate::upstream::ToolArgsType::String,
            {
                let state = Arc::clone(this);
                move |args| {
                    let essay = match args {
                        serde_json::Value::String(essay) => essay,
                        _ => {
                            return Err(
                                "Invalid argument, expected string".to_string()
                            );
                        }
                    };
                    if essay.trim().len() == 0 {
                        return Err("Essay cannot be empty".to_string());
                    }
                    let mut state = state.lock().unwrap();
                    state.essay = Some(essay);
                    Ok("Ok".to_string())
                }
            },
        )
    }

    pub fn validate_essay(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        let state = this.lock().unwrap();
        match &state.essay {
            Some(essay) => {
                if essay.trim().len() == 0 {
                    Err("Essay cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
            None => Err("Essay has not been written".to_string()),
        }
    }

    pub fn read_input_schema_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "ReadInputSchema",
            "Read Input Schema",
            crate::upstream::ToolArgsType::None,
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match &state.input_schema {
                        Some(input_schema) => {
                            Ok(serde_json::to_string(input_schema).unwrap())
                        }
                        None => {
                            Err("Input schema has not been written".to_string())
                        }
                    }
                }
            },
        )
    }

    pub fn write_input_schema_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "WriteInputSchema",
            "Write Input Schema",
            crate::upstream::ToolArgsType::Object,
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument, expected object: {}",
                                e
                            ));
                        }
                    };
                    let mut de = serde_json::Deserializer::from_str(&args_str);
                    let input_schema = match serde_path_to_error::deserialize::<
                        _,
                        functions::alpha_scalar::expression::ScalarFunctionInputSchema,
                    >(&mut de) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid input schema: {}",
                                e,
                            ));
                        }
                    };
                    let input_schema_wrapped =
                        functions::expression::InputSchema::Object(
                            input_schema,
                        );
                    match functions::check::check_input_schema(
                        &input_schema_wrapped,
                    ) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(
                                format!("Invalid input schema: {}", e,),
                            );
                        }
                    }
                    let input_schema = match input_schema_wrapped {
                        functions::expression::InputSchema::Object(o) => o,
                        _ => unreachable!(),
                    };
                    let mut state = state.lock().unwrap();
                    state.input_schema = Some(input_schema);
                    Ok("Ok".to_string())
                }
            },
        )
    }

    pub fn validate_input_schema(
        this: &Arc<Mutex<Self>>,
    ) -> Result<(), String> {
        let state = this.lock().unwrap();
        match &state.input_schema {
            Some(input_schema) => {
                let input_schema_wrapped =
                    functions::expression::InputSchema::Object(
                        input_schema.clone(),
                    );
                functions::check::check_input_schema(&input_schema_wrapped)
            }
            None => Err("Input schema has not been written".to_string()),
        }
    }

    pub fn read_essay_tasks_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "ReadEssayTasks",
            "Read Essay Tasks",
            crate::upstream::ToolArgsType::None,
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match &state.essay_tasks {
                        Some(essay_tasks) => Ok(essay_tasks.clone()),
                        None => {
                            Err("Essay tasks has not been written".to_string())
                        }
                    }
                }
            },
        )
    }

    pub fn write_essay_tasks_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "WriteEssayTasks",
            "Write Essay Tasks",
            crate::upstream::ToolArgsType::String,
            {
                let state = Arc::clone(this);
                move |args| {
                    let essay_tasks = match args {
                        serde_json::Value::String(essay_tasks) => essay_tasks,
                        _ => {
                            return Err(
                                "Invalid argument, expected string".to_string()
                            );
                        }
                    };
                    if essay_tasks.trim().len() == 0 {
                        return Err("Essay tasks cannot be empty".to_string());
                    }
                    let mut state = state.lock().unwrap();
                    state.essay_tasks = Some(essay_tasks);
                    Ok("Ok".to_string())
                }
            },
        )
    }

    pub fn validate_essay_tasks(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        let state = this.lock().unwrap();
        match &state.essay_tasks {
            Some(essay_tasks) => {
                if essay_tasks.trim().len() == 0 {
                    Err("Essay tasks cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
            None => Err("Essay tasks has not been written".to_string()),
        }
    }

    pub fn read_tasks_length_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "ReadTasksLength",
            "Read Tasks Length",
            crate::upstream::ToolArgsType::None,
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match &state.tasks {
                        Some(tasks) => Ok(tasks.len().to_string()),
                        None => Ok("0".to_string()),
                    }
                }
            },
        )
    }

    pub fn read_task_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "ReadTask",
            "Read Task by index",
            crate::upstream::ToolArgsType::Number,
            {
                let state = Arc::clone(this);
                move |args| {
                    let index = match args {
                        serde_json::Value::Number(n) => {
                            if let Some(u) = n.as_u64() {
                                u as usize
                            } else {
                                return Err(
                                    "Invalid argument, expected non-negative integer"
                                        .to_string()
                                );
                            }
                        }
                        _ => {
                            return Err(
                                "Invalid argument, expected number".to_string()
                            );
                        }
                    };
                    let state = state.lock().unwrap();
                    match &state.tasks {
                        Some(tasks) => {
                            if index < tasks.len() {
                                Ok(serde_json::to_string(&tasks[index])
                                    .unwrap())
                            } else {
                                Err("Index out of bounds".to_string())
                            }
                        }
                        None => Err("Tasks have not been written".to_string()),
                    }
                }
            },
        )
    }

    pub fn delete_task_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "DeleteTask",
            "Delete Task by index",
            crate::upstream::ToolArgsType::Number,
            {
                let state = Arc::clone(this);
                move |args| {
                    let index = match args {
                        serde_json::Value::Number(n) => {
                            if let Some(u) = n.as_u64() {
                                u as usize
                            } else {
                                return Err(
                                    "Invalid argument, expected non-negative integer"
                                        .to_string()
                                );
                            }
                        }
                        _ => {
                            return Err(
                                "Invalid argument, expected number".to_string()
                            );
                        }
                    };
                    let mut state = state.lock().unwrap();
                    match &mut state.tasks {
                        Some(tasks) => {
                            if index < tasks.len() {
                                tasks.remove(index);
                                Ok(tasks.len().to_string())
                            } else {
                                Err("Index out of bounds".to_string())
                            }
                        }
                        None => Err("Tasks have not been written".to_string()),
                    }
                }
            },
        )
    }

    pub fn append_task_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "AppendTask",
            "Append Task",
            crate::upstream::ToolArgsType::Object,
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument, expected object: {}",
                                e
                            ));
                        }
                    };
                    let task = match serde_path_to_error::deserialize::<
                        _,
                        functions::alpha_scalar::LeafTaskExpression,
                    >(
                        &mut serde_json::Deserializer::from_str(&args_str),
                    ) {
                        Ok(t) => t,
                        Err(e) => {
                            return Err(format!(
                                "Invalid task expression: {}",
                                e,
                            ));
                        }
                    };
                    let mut state = state.lock().unwrap();
                    match &mut state.tasks {
                        Some(tasks) => tasks.push(task),
                        None => state.tasks = Some(vec![task]),
                    }
                    Ok(state.tasks.as_ref().unwrap().len().to_string())
                }
            },
        )
    }

    pub fn check_function_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "CheckFunction",
            "Check if function is valid",
            crate::upstream::ToolArgsType::None,
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    let function =
                        functions::alpha_scalar::RemoteFunction::Leaf {
                            description: "placeholder".to_string(),
                            input_schema: state
                                .input_schema
                                .clone()
                                .ok_or_else(|| {
                                    "Input schema has not been written"
                                        .to_string()
                                })?,
                            tasks: state.tasks.clone().ok_or_else(|| {
                                "Tasks have not been written".to_string()
                            })?,
                        };
                    match functions::alpha_scalar::check::check_alpha_leaf_scalar_function(
                        &function,
                    ) {
                        Ok(_) => Ok("Function is valid".to_string()),
                        Err(e) => Err(format!("Function is invalid: {}", e)),
                    }
                }
            },
        )
    }

    pub fn validate_function(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        let state = this.lock().unwrap();
        let function = functions::alpha_scalar::RemoteFunction::Leaf {
            description: "placeholder".to_string(),
            input_schema: state.input_schema.clone().ok_or_else(|| {
                "Input schema has not been written".to_string()
            })?,
            tasks: state
                .tasks
                .clone()
                .ok_or_else(|| "Tasks have not been written".to_string())?,
        };
        functions::alpha_scalar::check::check_alpha_leaf_scalar_function(
            &function,
        )
    }

    pub fn read_description_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "ReadDescription",
            "Read Description",
            crate::upstream::ToolArgsType::None,
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match &state.description {
                        Some(description) => Ok(description.clone()),
                        None => {
                            Err("Description has not been written".to_string())
                        }
                    }
                }
            },
        )
    }

    pub fn write_description_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::upstream::Tool {
        crate::upstream::Tool::new_sync(
            "WriteDescription",
            "Write Description",
            crate::upstream::ToolArgsType::String,
            {
                let state = Arc::clone(this);
                move |args| {
                    let description = match args {
                        serde_json::Value::String(description) => description,
                        _ => {
                            return Err(
                                "Invalid argument, expected string".to_string()
                            );
                        }
                    };
                    if description.trim().len() == 0 {
                        return Err("Description cannot be empty".to_string());
                    } else if description.len() > 350 {
                        return Err(format!(
                            "Description is {} bytes, exceeds maximum of 350 bytes",
                            description.len(),
                        ));
                    }
                    let mut state = state.lock().unwrap();
                    state.description = Some(description);
                    Ok("Ok".to_string())
                }
            },
        )
    }

    pub fn validate_description(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        let state = this.lock().unwrap();
        match &state.description {
            Some(description) => {
                if description.trim().len() == 0 {
                    Err("Description cannot be empty".to_string())
                } else if description.len() > 350 {
                    Err(format!(
                        "Description is {} bytes, exceeds maximum of 350 bytes",
                        description.len(),
                    ))
                } else {
                    Ok(())
                }
            }
            None => Err("Description has not been written".to_string()),
        }
    }

    pub fn write_readme(this: &Arc<Mutex<Self>>) {
        let mut state = this.lock().unwrap();
        let description = match state.description.as_deref() {
            Some(description) => description,
            None => return,
        };
        state.readme = Some(super::readme::readme(
            &state.params.name,
            description,
            Vec::new(),
        ));
    }
}

impl super::InventionState for AlphaScalarLeafState {
    fn params(this: &Arc<Mutex<Self>>) -> super::Params {
        this.lock().unwrap().params.clone()
    }
    fn is_scalar() -> bool { true }
    fn object() -> crate::functions::inventions::response::streaming::Object {
        crate::functions::inventions::response::streaming::Object::AlphaScalarFunctionInventionChunk
    }
    fn into_state(self) -> super::State { super::State::AlphaScalarLeaf(self) }

    fn essay_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::upstream::Tool> {
        vec![Self::read_spec_tool(this), Self::write_essay_tool(this)]
    }
    fn validate_essay(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaScalarLeafState::validate_essay(this)
    }

    fn input_schema_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::upstream::Tool> {
        let mut tools = vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::write_input_schema_tool(this),
        ];
        tools.extend(crate::functions::inventions::schema_tools(&["ObjectInputSchema"]));
        tools
    }
    fn validate_input_schema(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaScalarLeafState::validate_input_schema(this)
    }

    fn essay_tasks_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::upstream::Tool> {
        vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::read_input_schema_tool(this), Self::write_essay_tasks_tool(this),
        ]
    }
    fn validate_essay_tasks(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaScalarLeafState::validate_essay_tasks(this)
    }

    fn tasks_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::upstream::Tool> {
        let mut tools = vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::read_input_schema_tool(this), Self::read_essay_tasks_tool(this),
            Self::append_task_tool(this), Self::delete_task_tool(this),
            Self::read_task_tool(this), Self::read_tasks_length_tool(this),
            Self::check_function_tool(this),
        ];
        tools.extend(crate::functions::inventions::schema_tools(&["AlphaScalarVectorCompletionTaskExpression", "Messages", "VectorResponses"]));
        tools
    }
    fn validate_function(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaScalarLeafState::validate_function(this)
    }

    fn description_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::upstream::Tool> {
        vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::read_input_schema_tool(this), Self::read_essay_tasks_tool(this),
            Self::read_task_tool(this), Self::read_tasks_length_tool(this),
            Self::write_description_tool(this),
        ]
    }
    fn validate_description(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaScalarLeafState::validate_description(this)
    }

    fn write_readme(this: &Arc<Mutex<Self>>) {
        AlphaScalarLeafState::write_readme(this)
    }
}
