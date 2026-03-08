use crate::functions;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaVectorLeafState {
    #[serde(flatten)]
    pub params: super::Params,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub essay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema:
        Option<functions::alpha_vector::expression::VectorFunctionInputSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub essay_tasks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<functions::alpha_vector::LeafTaskExpression>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
}

impl AlphaVectorLeafState {
    pub fn read_spec_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "ReadSpec",
            "Read Spec",
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "ReadEssay",
            "Read Essay",
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::functions::inventions::schema::EssayObject>(
            "WriteEssay",
            "Write Essay",
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e
                            ));
                        }
                    };
                    let parsed = match serde_path_to_error::deserialize::<
                        _,
                        crate::functions::inventions::schema::EssayObject,
                    >(
                        &mut serde_json::Deserializer::from_str(&args_str),
                    ) {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e,
                            ));
                        }
                    };
                    if parsed.essay.trim().len() == 0 {
                        return Err("Essay cannot be empty".to_string());
                    }
                    let mut state = state.lock().unwrap();
                    state.essay = Some(parsed.essay);
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "ReadInputSchema",
            "Read Input Schema",
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::AnyObjectJsonSchema>(
            "WriteInputSchema",
            "Write Input Schema",
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
                        functions::alpha_vector::expression::VectorFunctionInputSchema,
                    >(&mut de) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid input schema: {}",
                                e,
                            ));
                        }
                    };
                    let transpiled = input_schema.clone().transpile();
                    match functions::check::check_input_schema(&transpiled) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(
                                format!("Invalid input schema: {}", e,),
                            );
                        }
                    }
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
                let transpiled = input_schema.clone().transpile();
                functions::check::check_input_schema(&transpiled)
            }
            None => Err("Input schema has not been written".to_string()),
        }
    }

    pub fn read_essay_tasks_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "ReadEssayTasks",
            "Read Essay Tasks",
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::functions::inventions::schema::EssayTasksObject>(
            "WriteEssayTasks",
            "Write Essay Tasks",
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e
                            ));
                        }
                    };
                    let parsed = match serde_path_to_error::deserialize::<
                        _,
                        crate::functions::inventions::schema::EssayTasksObject,
                    >(
                        &mut serde_json::Deserializer::from_str(&args_str),
                    ) {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e,
                            ));
                        }
                    };
                    if parsed.essay_tasks.trim().len() == 0 {
                        return Err("Essay tasks cannot be empty".to_string());
                    }
                    let mut state = state.lock().unwrap();
                    state.essay_tasks = Some(parsed.essay_tasks);
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "ReadTasksLength",
            "Read Tasks Length",
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::functions::inventions::schema::IndexObject>(
            "ReadTask",
            "Read Task by index",
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e
                            ));
                        }
                    };
                    let parsed = match serde_path_to_error::deserialize::<
                        _,
                        crate::functions::inventions::schema::IndexObject,
                    >(
                        &mut serde_json::Deserializer::from_str(&args_str),
                    ) {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e,
                            ));
                        }
                    };
                    let index = parsed.index as usize;
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::functions::inventions::schema::IndexObject>(
            "DeleteTask",
            "Delete Task by index",
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e
                            ));
                        }
                    };
                    let parsed = match serde_path_to_error::deserialize::<
                        _,
                        crate::functions::inventions::schema::IndexObject,
                    >(
                        &mut serde_json::Deserializer::from_str(&args_str),
                    ) {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e,
                            ));
                        }
                    };
                    let index = parsed.index as usize;
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::AnyObjectJsonSchema>(
            "AppendTask",
            "Append Task",
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
                        functions::alpha_vector::LeafTaskExpression,
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "CheckFunction",
            "Check if function is valid",
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    let function =
                        functions::alpha_vector::RemoteFunction::Leaf {
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
                    match functions::alpha_vector::check::check_alpha_leaf_vector_function(
                        &function,
                    ) {
                        Ok(_) => Ok("Function is valid".to_string()),
                        Err(e) => Err(format!("Function is invalid: {}", e)),
                    }
                }
            },
        )
    }

    pub fn read_predicted_tasks_length_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "ReadPredictedTasksLength",
            "Read Predicted Tasks Length",
            {
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match state.tasks_length {
                        Some(n) => Ok(n.to_string()),
                        None => Err("Predicted tasks length has not been set".to_string()),
                    }
                }
            },
        )
    }

    pub fn edit_predicted_tasks_length_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::functions::inventions::schema::TasksLengthObject>(
            "EditPredictedTasksLength",
            "Edit Predicted Tasks Length",
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e
                            ));
                        }
                    };
                    let parsed = match serde_path_to_error::deserialize::<
                        _,
                        crate::functions::inventions::schema::TasksLengthObject,
                    >(
                        &mut serde_json::Deserializer::from_str(&args_str),
                    ) {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e,
                            ));
                        }
                    };
                    let mut guard = state.lock().unwrap();
                    let min = guard.params.min_leaf_width;
                    let max = guard.params.max_leaf_width;
                    if parsed.tasks_length < min || parsed.tasks_length > max {
                        return Err(format!(
                            "Tasks length {} is outside allowed range [{}, {}]",
                            parsed.tasks_length, min, max,
                        ));
                    }
                    guard.tasks_length = Some(parsed.tasks_length);
                    Ok("Ok".to_string())
                }
            },
        )
    }

    pub fn validate_function(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        let state = this.lock().unwrap();
        let tasks_length = state.tasks_length.ok_or_else(|| {
            "Tasks length has not been set".to_string()
        })?;
        let actual_len = state.tasks.as_ref().map(|t| t.len()).unwrap_or(0) as u64;
        if tasks_length != actual_len {
            return Err(format!(
                "Tasks length {} does not match actual tasks length {}",
                tasks_length, actual_len,
            ));
        }
        let function = functions::alpha_vector::RemoteFunction::Leaf {
            description: "placeholder".to_string(),
            input_schema: state.input_schema.clone().ok_or_else(|| {
                "Input schema has not been written".to_string()
            })?,
            tasks: state
                .tasks
                .clone()
                .ok_or_else(|| "Tasks have not been written".to_string())?,
        };
        functions::alpha_vector::check::check_alpha_leaf_vector_function(
            &function,
        )
    }

    pub fn read_description_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::json_schema::EmptyObjectJsonSchema>(
            "ReadDescription",
            "Read Description",
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
    ) -> crate::functions::inventions::InventionTool {
        crate::functions::inventions::InventionTool::new_sync::<crate::functions::inventions::schema::DescriptionObject>(
            "WriteDescription",
            "Write Description",
            {
                let state = Arc::clone(this);
                move |args| {
                    let args_str = match serde_json::to_string(&args) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e
                            ));
                        }
                    };
                    let parsed = match serde_path_to_error::deserialize::<
                        _,
                        crate::functions::inventions::schema::DescriptionObject,
                    >(
                        &mut serde_json::Deserializer::from_str(&args_str),
                    ) {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(format!(
                                "Invalid argument: {}",
                                e,
                            ));
                        }
                    };
                    if parsed.description.trim().len() == 0 {
                        return Err("Description cannot be empty".to_string());
                    } else if parsed.description.len() > 350 {
                        return Err(format!(
                            "Description is {} bytes, exceeds maximum of 350 bytes",
                            parsed.description.len(),
                        ));
                    }
                    let mut state = state.lock().unwrap();
                    state.description = Some(parsed.description);
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

impl super::InventionState for AlphaVectorLeafState {
    fn params(this: &Arc<Mutex<Self>>) -> super::Params {
        this.lock().unwrap().params.clone()
    }
    fn is_scalar() -> bool { false }
    fn object() -> crate::functions::inventions::response::streaming::Object {
        crate::functions::inventions::response::streaming::Object::AlphaVectorFunctionInventionChunk
    }
    fn into_state(self) -> super::State { super::State::AlphaVectorLeaf(self) }

    fn essay_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::functions::inventions::InventionTool> {
        vec![Self::read_spec_tool(this), Self::write_essay_tool(this)]
    }
    fn validate_essay(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaVectorLeafState::validate_essay(this)
    }

    fn input_schema_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::functions::inventions::InventionTool> {
        let mut tools = vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::write_input_schema_tool(this),
        ];
        tools.extend(crate::functions::inventions::schema_tools(&["AlphaVectorFunctionInputSchema"]));
        tools
    }
    fn validate_input_schema(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaVectorLeafState::validate_input_schema(this)
    }

    fn essay_tasks_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::functions::inventions::InventionTool> {
        vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::read_input_schema_tool(this), Self::write_essay_tasks_tool(this),
        ]
    }
    fn validate_essay_tasks(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaVectorLeafState::validate_essay_tasks(this)
    }

    fn tasks_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::functions::inventions::InventionTool> {
        let mut tools = vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::read_input_schema_tool(this), Self::read_essay_tasks_tool(this),
            Self::append_task_tool(this), Self::delete_task_tool(this),
            Self::read_task_tool(this), Self::read_tasks_length_tool(this),
            Self::check_function_tool(this),
            Self::read_predicted_tasks_length_tool(this),
            Self::edit_predicted_tasks_length_tool(this),
        ];
        tools.extend(crate::functions::inventions::schema_tools(&["AlphaVectorVectorCompletionTaskExpression", "Messages", "VectorResponses"]));
        tools
    }
    fn validate_function(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaVectorLeafState::validate_function(this)
    }
    fn build_function(this: &Arc<Mutex<Self>>) -> Option<crate::functions::FullRemoteFunction> {
        let state = this.lock().unwrap();
        Some(crate::functions::FullRemoteFunction::Alpha(
            crate::functions::AlphaRemoteFunction::Vector(
                crate::functions::alpha_vector::RemoteFunction::Leaf {
                    description: state.description.clone()?,
                    input_schema: state.input_schema.clone()?,
                    tasks: state.tasks.clone()?,
                },
            ),
        ))
    }

    fn description_tools(this: &Arc<Mutex<Self>>) -> Vec<crate::functions::inventions::InventionTool> {
        vec![
            Self::read_spec_tool(this), Self::read_essay_tool(this),
            Self::read_input_schema_tool(this), Self::read_essay_tasks_tool(this),
            Self::read_task_tool(this), Self::read_tasks_length_tool(this),
            Self::write_description_tool(this),
        ]
    }
    fn validate_description(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        AlphaVectorLeafState::validate_description(this)
    }

    fn write_readme(this: &Arc<Mutex<Self>>) {
        AlphaVectorLeafState::write_readme(this)
    }
}
