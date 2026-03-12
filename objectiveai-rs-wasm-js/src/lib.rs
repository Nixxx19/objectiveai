//! WebAssembly bindings for ObjectiveAI.
//!
//! This crate provides JavaScript/TypeScript bindings for client-side validation
//! and compilation of ObjectiveAI types. It enables browser-based applications to:
//!
//! - Validate Ensemble LLM and Ensemble configurations
//! - Compute content-addressed IDs (deterministic hashes)
//! - Compile Function expressions for previewing during authoring
//! - Compute prompt, tools, and response IDs for caching/deduplication
//!
//! # Usage
//!
//! This crate is compiled to WebAssembly and consumed via the `objectiveai` npm package.
//! The TypeScript SDK wraps these functions with proper type definitions.
//!
//! # Functions
//!
//! - [`validateAgent`] - Validate and compute ID for an Agent
//! - [`validateEnsemble`] - Validate and compute ID for an Ensemble
//! - [`compileFunctionTasks`] - Compile function tasks for a given input
//! - [`compileFunctionOutput`] - Compile function output from task results
//! - [`promptId`] - Compute content-addressed ID for chat messages
//! - [`vectorResponseId`] - Compute content-addressed ID for a response option

#![allow(non_snake_case)]
use wasm_bindgen::prelude::*;

/// Validates an Agent configuration and computes its content-addressed ID.
///
/// Takes an Agent definition, normalizes it (removes defaults, deduplicates),
/// validates all fields, and computes a deterministic ID using XXHash3-128.
///
/// # Arguments
///
/// * `agent` - JavaScript object representing an Agent configuration
///
/// # Returns
///
/// The validated Agent with its computed `id` field populated.
///
/// # Errors
///
/// Returns an error string if validation fails (e.g., invalid model name,
/// out-of-range parameters, conflicting settings).
#[wasm_bindgen]
pub fn validateAgent(agent: JsValue) -> Result<JsValue, JsValue> {
    // deserialize
    let agent_base: objectiveai::agent::AgentBase =
        serde_wasm_bindgen::from_value(agent)?;
    // prepare, validate, and compute ID
    let agent: objectiveai::agent::Agent = agent_base
        .try_into()
        .map_err(|e: String| JsValue::from_str(&e))?;
    // serialize
    let agent: JsValue = serde_wasm_bindgen::to_value(&agent)?;
    Ok(agent)
}

/// Validates an Ensemble configuration and computes its content-addressed ID.
///
/// Takes an Ensemble definition (a collection of Ensemble LLMs), validates each
/// LLM, and computes a deterministic ID for the ensemble as a whole.
///
/// # Arguments
///
/// * `ensemble` - JavaScript object representing an Ensemble configuration
///
/// # Returns
///
/// The validated Ensemble with its computed `id` field populated and all
/// member LLMs validated with their IDs.
///
/// # Errors
///
/// Returns an error string if any LLM validation fails or the ensemble
/// structure is invalid.
#[wasm_bindgen]
pub fn validateEnsemble(ensemble: JsValue) -> Result<JsValue, JsValue> {
    // deserialize
    let ensemble_base: objectiveai::ensemble::EnsembleBase =
        serde_wasm_bindgen::from_value(ensemble)?;
    // prepare, validate, and compute ID
    let ensemble: objectiveai::ensemble::Ensemble = ensemble_base
        .try_into()
        .map_err(|e: String| JsValue::from_str(&e))?;
    // serialize
    let ensemble: JsValue = serde_wasm_bindgen::to_value(&ensemble)?;
    Ok(ensemble)
}

/// Validates function input against its schema.
///
/// For remote functions, checks whether the provided input conforms to
/// the function's JSON Schema definition. For inline functions, returns
/// `null` since they lack schema definitions.
///
/// # Arguments
///
/// * `function` - JavaScript object representing a Function definition
/// * `input` - JavaScript object representing the function input to validate
///
/// # Returns
///
/// - `true` if the input is valid against the schema
/// - `false` if the input is invalid
/// - `null` for inline functions (no schema to validate against)
///
/// # Errors
///
/// Returns an error if deserialization fails.
#[wasm_bindgen]
pub fn validateFunctionInput(
    function: JsValue,
    input: JsValue,
) -> Result<Option<bool>, JsValue> {
    // deserialize
    let function: objectiveai::functions::Function =
        serde_wasm_bindgen::from_value(function)?;
    let input: objectiveai::functions::expression::Input =
        serde_wasm_bindgen::from_value(input)?;
    // validate input
    Ok(function.validate_input(&input))
}

/// Compiles a Function's task expressions for a given input.
///
/// Evaluates all expressions (JMESPath or Starlark) in the function's tasks
/// using the provided input data. This is used for previewing how tasks will
/// be executed during Function authoring.
///
/// # Arguments
///
/// * `function` - JavaScript object representing a Function definition
/// * `input` - JavaScript object representing the function input
///
/// # Returns
///
/// An array where each element corresponds to a task definition:
/// - `null` if the task was skipped (skip expression evaluated to true)
/// - `{ One: task }` for non-mapped tasks
/// - `{ Many: [task, ...] }` for mapped tasks (expanded from map expression)
///
/// # Errors
///
/// Returns an error string if expression evaluation fails or types don't match.
#[wasm_bindgen]
pub fn compileFunctionTasks(
    function: JsValue,
    input: JsValue,
) -> Result<JsValue, JsValue> {
    // deserialize
    let function: objectiveai::functions::Function =
        serde_wasm_bindgen::from_value(function)?;
    let input: objectiveai::functions::expression::Input =
        serde_wasm_bindgen::from_value(input)?;
    // compile tasks
    let tasks = function
        .compile_tasks(&input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    // serialize
    let tasks: JsValue = serde_wasm_bindgen::to_value(&tasks)?;
    Ok(tasks)
}

// TODO: Update for new per-task output expression architecture
// /// Computes the final output of a Function given input and task results.
// ///
// /// Evaluates the function's output expression using the provided input data
// /// and task outputs. Also validates that the output meets constraints:
// /// - Scalar functions: output must be in [0, 1]
// /// - Vector functions: output must sum to approximately 1
// ///
// /// # Arguments
// ///
// /// * `function` - JavaScript object representing a Function definition
// /// * `input` - JavaScript object representing the function input
// /// * `task_outputs` - Array of task outputs (from actual execution or mocked)
// ///
// /// # Returns
// ///
// /// An object with:
// /// - `output`: The computed scalar or vector output
// /// - `valid`: Boolean indicating if the output meets constraints
// ///
// /// # Errors
// ///
// /// Returns an error string if expression evaluation fails.
// #[wasm_bindgen]
// pub fn compileFunctionOutput(
//     function: JsValue,
//     input: JsValue,
//     task_outputs: JsValue,
// ) -> Result<JsValue, JsValue> {
//     // deserialize
//     let function: objectiveai::functions::Function =
//         serde_wasm_bindgen::from_value(function)?;
//     let input: objectiveai::functions::expression::Input =
//         serde_wasm_bindgen::from_value(input)?;
//     let task_outputs: Vec<
//         Option<objectiveai::functions::expression::TaskOutput<'static>>,
//     > = serde_wasm_bindgen::from_value(task_outputs)?;
//     // compile output
//     let output = function
//         .compile_output(&input, &task_outputs)
//         .map_err(|e| JsValue::from_str(&e.to_string()))?;
//     // serialize
//     let output: JsValue = serde_wasm_bindgen::to_value(&output)?;
//     Ok(output)
// }

/// Computes the expected output length for a vector Function.
///
/// Evaluates the `output_length` expression to determine how many elements
/// the output vector should contain. This is only applicable to remote
/// vector functions which have an `output_length` field.
///
/// # Arguments
///
/// * `function` - JavaScript object representing a Function definition
/// * `input` - JavaScript object representing the function input
///
/// # Returns
///
/// - The expected output length for remote vector functions
/// - `null` for scalar functions or inline functions
///
/// # Errors
///
/// Returns an error string if expression evaluation fails.
#[wasm_bindgen]
pub fn compileFunctionOutputLength(
    function: JsValue,
    input: JsValue,
) -> Result<Option<u32>, JsValue> {
    // deserialize
    let function: objectiveai::functions::Function =
        serde_wasm_bindgen::from_value(function)?;
    let input: objectiveai::functions::expression::Input =
        serde_wasm_bindgen::from_value(input)?;
    // compile output length
    Ok(function
        .compile_output_length(&input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .map(|u| u as u32))
}

/// Compiles the `input_split` expression to split input into multiple sub-inputs.
///
/// Used by strategies like Swiss System that need to partition input into
/// smaller pools. The expression transforms the original input into an array
/// of inputs, where each element can be processed independently.
///
/// # Arguments
///
/// * `function` - JavaScript object representing a Function definition
/// * `input` - JavaScript object representing the function input to split
///
/// # Returns
///
/// - An array of split inputs for vector functions with `input_split` defined
/// - `null` for scalar functions or functions without `input_split`
///
/// # Errors
///
/// Returns an error string if expression evaluation fails.
#[wasm_bindgen]
pub fn compileFunctionInputSplit(
    function: JsValue,
    input: JsValue,
) -> Result<Option<JsValue>, JsValue> {
    // deserialize
    let function: objectiveai::functions::Function =
        serde_wasm_bindgen::from_value(function)?;
    let input: objectiveai::functions::expression::Input =
        serde_wasm_bindgen::from_value(input)?;
    // compile input split
    let input_split = function
        .compile_input_split(&input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    // serialize
    let input_split: Option<JsValue> = input_split
        .map(|split| serde_wasm_bindgen::to_value(&split))
        .transpose()?;
    Ok(input_split)
}

/// Compiles the `input_merge` expression to merge multiple sub-inputs back into one.
///
/// Used by strategies like Swiss System to recombine a subset of split inputs
/// into a single input for pool execution. The expression transforms an array
/// of inputs (a subset from `compileFunctionInputSplit`) into a single merged input.
///
/// # Arguments
///
/// * `function` - JavaScript object representing a Function definition
/// * `input` - Array of inputs to merge (typically a subset from `compileFunctionInputSplit`)
///
/// # Returns
///
/// - The merged input for vector functions with `input_merge` defined
/// - `null` for scalar functions or functions without `input_merge`
///
/// # Errors
///
/// Returns an error string if expression evaluation fails.
#[wasm_bindgen]
pub fn compileFunctionInputMerge(
    function: JsValue,
    input: JsValue,
) -> Result<Option<JsValue>, JsValue> {
    // deserialize
    let function: objectiveai::functions::Function =
        serde_wasm_bindgen::from_value(function)?;
    let input: Vec<objectiveai::functions::expression::Input> =
        serde_wasm_bindgen::from_value(input)?;
    // compile input merge
    let input_merge = function
        .compile_input_merge(&objectiveai::functions::expression::Input::Array(
            input,
        ))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    // serialize
    let input_merge: Option<JsValue> = input_merge
        .map(|merge| serde_wasm_bindgen::to_value(&merge))
        .transpose()?;
    Ok(input_merge)
}

/// Validates vector function fields (output_length, input_split, input_merge).
///
/// Generates diverse example inputs from the input_schema and validates that the
/// output_length, input_split, and input_merge expressions work correctly together
/// via round-trip testing.
#[wasm_bindgen]
pub fn checkVectorFields(fields: JsValue) -> Result<(), JsValue> {
    let fields: objectiveai::functions::check::VectorFieldsValidation =
        serde_wasm_bindgen::from_value(fields)?;
    objectiveai::functions::check::check_vector_fields(fields)
        .map_err(|e| JsValue::from_str(&e))
}

/// Validates scalar function fields (input_schema only).
#[wasm_bindgen]
pub fn checkScalarFields(fields: JsValue) -> Result<(), JsValue> {
    let fields: objectiveai::functions::check::ScalarFieldsValidation =
        serde_wasm_bindgen::from_value(fields)?;
    objectiveai::functions::check::check_scalar_fields(fields)
        .map_err(|e| JsValue::from_str(&e))
}

/// Alpha check for a leaf scalar function (depth 0, scalar output).
#[wasm_bindgen]
pub fn alphaCheckLeafScalarFunction(function: JsValue) -> Result<(), JsValue> {
    let function: objectiveai::functions::alpha_scalar::RemoteFunction =
        serde_wasm_bindgen::from_value(function)?;
    objectiveai::functions::alpha_scalar::check::check_alpha_leaf_scalar_function(&function)
        .map_err(|e| JsValue::from_str(&e))
}

/// Alpha check for a branch scalar function (depth > 0, scalar output).
///
/// `children` is an optional map of child function name → RemoteFunction for
/// validating placeholder task inputs against child function input schemas.
#[wasm_bindgen]
pub fn alphaCheckBranchScalarFunction(function: JsValue, children: JsValue) -> Result<(), JsValue> {
    let function: objectiveai::functions::alpha_scalar::RemoteFunction =
        serde_wasm_bindgen::from_value(function)?;
    let children: Option<std::collections::HashMap<String, objectiveai::functions::RemoteFunction>> =
        if children.is_undefined() || children.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(children)?)
        };
    objectiveai::functions::alpha_scalar::check::check_alpha_branch_scalar_function(&function, children.as_ref())
        .map_err(|e| JsValue::from_str(&e))
}

/// Alpha check for a leaf vector function (depth 0, vector output).
#[wasm_bindgen]
pub fn alphaCheckLeafVectorFunction(function: JsValue) -> Result<(), JsValue> {
    let function: objectiveai::functions::alpha_vector::RemoteFunction =
        serde_wasm_bindgen::from_value(function)?;
    objectiveai::functions::alpha_vector::check::check_alpha_leaf_vector_function(&function)
        .map_err(|e| JsValue::from_str(&e))
}

/// Alpha check for a branch vector function (depth > 0, vector output).
///
/// `children` is an optional map of child function name → RemoteFunction for
/// validating placeholder task inputs against child function input schemas.
#[wasm_bindgen]
pub fn alphaCheckBranchVectorFunction(function: JsValue, children: JsValue) -> Result<(), JsValue> {
    let function: objectiveai::functions::alpha_vector::RemoteFunction =
        serde_wasm_bindgen::from_value(function)?;
    let children: Option<std::collections::HashMap<String, objectiveai::functions::RemoteFunction>> =
        if children.is_undefined() || children.is_null() {
            None
        } else {
            Some(serde_wasm_bindgen::from_value(children)?)
        };
    objectiveai::functions::alpha_vector::check::check_alpha_branch_vector_function(&function, children.as_ref())
        .map_err(|e| JsValue::from_str(&e))
}

/// Computes a content-addressed ID for chat messages.
///
/// Normalizes the messages (consolidates text parts, removes empty content)
/// and computes a deterministic hash. This ID is used for caching and
/// deduplicating requests with identical prompts.
///
/// # Arguments
///
/// * `prompt` - Array of chat messages
///
/// # Returns
///
/// A base62-encoded hash string uniquely identifying the prompt content.
///
/// # Errors
///
/// Returns an error if the messages cannot be deserialized.
#[wasm_bindgen]
pub fn promptId(prompt: JsValue) -> Result<String, JsValue> {
    // deserialize
    let mut prompt: Vec<objectiveai::agent::completions::message::Message> =
        serde_wasm_bindgen::from_value(prompt)?;
    // prepare and compute ID
    objectiveai::agent::completions::message::prompt::prepare(&mut prompt);
    let id = objectiveai::agent::completions::message::prompt::id(&prompt);
    Ok(id)
}

/// Computes a content-addressed ID for a vector completion response option.
///
/// Normalizes the response content (consolidates text parts, removes empty
/// content) and computes a deterministic hash. This ID is used for caching
/// and identifying individual response options in vector completions.
///
/// # Arguments
///
/// * `response` - A rich content object (text or multipart content)
///
/// # Returns
///
/// A base62-encoded hash string uniquely identifying the response content.
///
/// # Errors
///
/// Returns an error if the response cannot be deserialized.
#[wasm_bindgen]
pub fn vectorResponseId(response: JsValue) -> Result<String, JsValue> {
    // deserialize
    let mut response: objectiveai::agent::completions::message::RichContent =
        serde_wasm_bindgen::from_value(response)?;
    // prepare and compute ID
    response.prepare();
    let id = response.id();
    Ok(id)
}

/// Merges two `AgentCompletionChunk`s and returns the merged result.
#[wasm_bindgen]
pub fn agentCompletionChunkMerged(a: JsValue, b: JsValue) -> Result<JsValue, JsValue> {
    let mut a: objectiveai::agent::completions::response::streaming::AgentCompletionChunk =
        serde_wasm_bindgen::from_value(a)?;
    let b: objectiveai::agent::completions::response::streaming::AgentCompletionChunk =
        serde_wasm_bindgen::from_value(b)?;
    a.push(&b);
    Ok(serde_wasm_bindgen::to_value(&a)?)
}

/// Merges two `VectorCompletionChunk`s and returns the merged result.
#[wasm_bindgen]
pub fn vectorCompletionChunkMerged(a: JsValue, b: JsValue) -> Result<JsValue, JsValue> {
    let mut a: objectiveai::vector::completions::response::streaming::VectorCompletionChunk =
        serde_wasm_bindgen::from_value(a)?;
    let b: objectiveai::vector::completions::response::streaming::VectorCompletionChunk =
        serde_wasm_bindgen::from_value(b)?;
    a.push(&b);
    Ok(serde_wasm_bindgen::to_value(&a)?)
}

/// Merges two `FunctionExecutionChunk`s and returns the merged result.
#[wasm_bindgen]
pub fn functionExecutionChunkMerged(a: JsValue, b: JsValue) -> Result<JsValue, JsValue> {
    let mut a: objectiveai::functions::executions::response::streaming::FunctionExecutionChunk =
        serde_wasm_bindgen::from_value(a)?;
    let b: objectiveai::functions::executions::response::streaming::FunctionExecutionChunk =
        serde_wasm_bindgen::from_value(b)?;
    a.push(&b);
    Ok(serde_wasm_bindgen::to_value(&a)?)
}

/// Merges two `FunctionInventionChunk`s and returns the merged result.
#[wasm_bindgen]
pub fn functionInventionChunkMerged(a: JsValue, b: JsValue) -> Result<JsValue, JsValue> {
    let mut a: objectiveai::functions::inventions::response::streaming::FunctionInventionChunk =
        serde_wasm_bindgen::from_value(a)?;
    let b: objectiveai::functions::inventions::response::streaming::FunctionInventionChunk =
        serde_wasm_bindgen::from_value(b)?;
    a.push(&b);
    Ok(serde_wasm_bindgen::to_value(&a)?)
}

/// Merges two `FunctionInventionRecursiveChunk`s and returns the merged result.
#[wasm_bindgen]
pub fn functionInventionRecursiveChunkMerged(a: JsValue, b: JsValue) -> Result<JsValue, JsValue> {
    let mut a: objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk =
        serde_wasm_bindgen::from_value(a)?;
    let b: objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk =
        serde_wasm_bindgen::from_value(b)?;
    a.push(&b);
    Ok(serde_wasm_bindgen::to_value(&a)?)
}

/// Merges two `FunctionProfileComputationChunk`s and returns the merged result.
#[wasm_bindgen]
pub fn functionProfileComputationChunkMerged(a: JsValue, b: JsValue) -> Result<JsValue, JsValue> {
    let mut a: objectiveai::functions::profiles::computations::response::streaming::FunctionProfileComputationChunk =
        serde_wasm_bindgen::from_value(a)?;
    let b: objectiveai::functions::profiles::computations::response::streaming::FunctionProfileComputationChunk =
        serde_wasm_bindgen::from_value(b)?;
    a.push(&b);
    Ok(serde_wasm_bindgen::to_value(&a)?)
}
