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
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
}

impl AlphaVectorLeafState {
    pub fn read_spec_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "ReadSpec",
            description: "Read Spec",
            args_type: crate::functions::inventions::ToolArgsType::None,
            call: Arc::new({
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    Ok(state.params.spec.clone())
                }
            }),
        }
    }

    pub fn read_essay_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "ReadEssay",
            description: "Read Essay",
            args_type: crate::functions::inventions::ToolArgsType::None,
            call: Arc::new({
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match &state.essay {
                        Some(essay) => Ok(essay.clone()),
                        None => Err("Essay has not been written".to_string()),
                    }
                }
            }),
        }
    }

    pub fn write_essay_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "WriteEssay",
            description: "Write Essay",
            args_type: crate::functions::inventions::ToolArgsType::String,
            call: Arc::new({
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
            }),
        }
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
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "ReadInputSchema",
            description: "Read Input Schema",
            args_type: crate::functions::inventions::ToolArgsType::None,
            call: Arc::new({
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
            }),
        }
    }

    pub fn write_input_schema_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "WriteInputSchema",
            description: "Write Input Schema",
            args_type: crate::functions::inventions::ToolArgsType::Object,
            call: Arc::new({
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
            }),
        }
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
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "ReadEssayTasks",
            description: "Read Essay Tasks",
            args_type: crate::functions::inventions::ToolArgsType::None,
            call: Arc::new({
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
            }),
        }
    }

    pub fn write_essay_tasks_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "WriteEssayTasks",
            description: "Write Essay Tasks",
            args_type: crate::functions::inventions::ToolArgsType::String,
            call: Arc::new({
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
            }),
        }
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
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "ReadTasksLength",
            description: "Read Tasks Length",
            args_type: crate::functions::inventions::ToolArgsType::None,
            call: Arc::new({
                let state = Arc::clone(this);
                move |_| {
                    let state = state.lock().unwrap();
                    match &state.tasks {
                        Some(tasks) => Ok(tasks.len().to_string()),
                        None => Ok("0".to_string()),
                    }
                }
            }),
        }
    }

    pub fn read_task_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "ReadTask",
            description: "Read Task by index",
            args_type: crate::functions::inventions::ToolArgsType::Number,
            call: Arc::new({
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
            }),
        }
    }

    pub fn delete_task_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "DeleteTask",
            description: "Delete Task by index",
            args_type: crate::functions::inventions::ToolArgsType::Number,
            call: Arc::new({
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
            }),
        }
    }

    pub fn append_task_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "AppendTask",
            description: "Append Task",
            args_type: crate::functions::inventions::ToolArgsType::Object,
            call: Arc::new({
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
            }),
        }
    }

    pub fn check_function_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "CheckFunction",
            description: "Check if function is valid",
            args_type: crate::functions::inventions::ToolArgsType::None,
            call: Arc::new({
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
            }),
        }
    }

    pub fn validate_function(this: &Arc<Mutex<Self>>) -> Result<(), String> {
        let state = this.lock().unwrap();
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
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "ReadDescription",
            description: "Read Description",
            args_type: crate::functions::inventions::ToolArgsType::None,
            call: Arc::new({
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
            }),
        }
    }

    pub fn write_description_tool(
        this: &Arc<Mutex<Self>>,
    ) -> crate::functions::inventions::Tool {
        crate::functions::inventions::Tool {
            name: "WriteDescription",
            description: "Write Description",
            args_type: crate::functions::inventions::ToolArgsType::String,
            call: Arc::new({
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
            }),
        }
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
