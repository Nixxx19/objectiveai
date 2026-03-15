//go:build cgo

package objectiveai

// #cgo CFLAGS: -I${SRCDIR}/../objectiveai-rs-cffi/include
// #include "objectiveai.h"
// #include <stdlib.h>
import "C"

import (
	"encoding/json"
	"errors"
	"unsafe"
)

// callCFFI1 marshals one input, calls a single-input C function, and unmarshals the result.
func callCFFI1[In any, Out any](
	input In,
	cfn func(*C.uint8_t, C.size_t, **C.uint8_t, *C.size_t) C.int32_t,
) (*Out, error) {
	jsonIn, err := json.Marshal(input)
	if err != nil {
		return nil, err
	}

	var outPtr *C.uint8_t
	var outLen C.size_t

	rc := cfn(
		(*C.uint8_t)(unsafe.Pointer(&jsonIn[0])),
		C.size_t(len(jsonIn)),
		&outPtr, &outLen,
	)

	result := C.GoBytes(unsafe.Pointer(outPtr), C.int(outLen))
	C.objectiveai_free(outPtr, outLen)

	if rc != 0 {
		return nil, errors.New(string(result))
	}

	var out Out
	if err := json.Unmarshal(result, &out); err != nil {
		return nil, err
	}
	return &out, nil
}

// callCFFI1Bytes marshals one input, calls a single-input C function, returns raw bytes (for string results).
func callCFFI1Bytes[In any](
	input In,
	cfn func(*C.uint8_t, C.size_t, **C.uint8_t, *C.size_t) C.int32_t,
) ([]byte, error) {
	jsonIn, err := json.Marshal(input)
	if err != nil {
		return nil, err
	}

	var outPtr *C.uint8_t
	var outLen C.size_t

	rc := cfn(
		(*C.uint8_t)(unsafe.Pointer(&jsonIn[0])),
		C.size_t(len(jsonIn)),
		&outPtr, &outLen,
	)

	result := C.GoBytes(unsafe.Pointer(outPtr), C.int(outLen))
	C.objectiveai_free(outPtr, outLen)

	if rc != 0 {
		return nil, errors.New(string(result))
	}
	return result, nil
}

// callCFFI1Void marshals one input, calls a single-input C function, expects no output on success.
func callCFFI1Void[In any](
	input In,
	cfn func(*C.uint8_t, C.size_t, **C.uint8_t, *C.size_t) C.int32_t,
) error {
	jsonIn, err := json.Marshal(input)
	if err != nil {
		return err
	}

	var outPtr *C.uint8_t
	var outLen C.size_t

	rc := cfn(
		(*C.uint8_t)(unsafe.Pointer(&jsonIn[0])),
		C.size_t(len(jsonIn)),
		&outPtr, &outLen,
	)

	if rc != 0 {
		result := C.GoBytes(unsafe.Pointer(outPtr), C.int(outLen))
		C.objectiveai_free(outPtr, outLen)
		return errors.New(string(result))
	}
	C.objectiveai_free(outPtr, outLen)
	return nil
}

// callCFFI2 marshals two inputs, calls a two-input C function, and unmarshals the result.
func callCFFI2[In1 any, In2 any, Out any](
	input1 In1, input2 In2,
	cfn func(*C.uint8_t, C.size_t, *C.uint8_t, C.size_t, **C.uint8_t, *C.size_t) C.int32_t,
) (*Out, error) {
	jsonIn1, err := json.Marshal(input1)
	if err != nil {
		return nil, err
	}
	jsonIn2, err := json.Marshal(input2)
	if err != nil {
		return nil, err
	}

	var outPtr *C.uint8_t
	var outLen C.size_t

	rc := cfn(
		(*C.uint8_t)(unsafe.Pointer(&jsonIn1[0])),
		C.size_t(len(jsonIn1)),
		(*C.uint8_t)(unsafe.Pointer(&jsonIn2[0])),
		C.size_t(len(jsonIn2)),
		&outPtr, &outLen,
	)

	result := C.GoBytes(unsafe.Pointer(outPtr), C.int(outLen))
	C.objectiveai_free(outPtr, outLen)

	if rc != 0 {
		return nil, errors.New(string(result))
	}

	var out Out
	if err := json.Unmarshal(result, &out); err != nil {
		return nil, err
	}
	return &out, nil
}

// callCFFI2Void marshals two inputs, calls a two-input C function, expects no output on success.
func callCFFI2Void[In1 any, In2 any](
	input1 In1, input2 In2,
	cfn func(*C.uint8_t, C.size_t, *C.uint8_t, C.size_t, **C.uint8_t, *C.size_t) C.int32_t,
) error {
	jsonIn1, err := json.Marshal(input1)
	if err != nil {
		return err
	}
	jsonIn2, err := json.Marshal(input2)
	if err != nil {
		return err
	}

	var outPtr *C.uint8_t
	var outLen C.size_t

	rc := cfn(
		(*C.uint8_t)(unsafe.Pointer(&jsonIn1[0])),
		C.size_t(len(jsonIn1)),
		(*C.uint8_t)(unsafe.Pointer(&jsonIn2[0])),
		C.size_t(len(jsonIn2)),
		&outPtr, &outLen,
	)

	if rc != 0 {
		result := C.GoBytes(unsafe.Pointer(outPtr), C.int(outLen))
		C.objectiveai_free(outPtr, outLen)
		return errors.New(string(result))
	}
	C.objectiveai_free(outPtr, outLen)
	return nil
}

// ---------------------------------------------------------------------------
// Memory Management
// ---------------------------------------------------------------------------

// Free releases memory allocated by the Rust FFI layer.
func Free(ptr *C.uint8_t, len C.size_t) {
	C.objectiveai_free(ptr, len)
}

// ---------------------------------------------------------------------------
// Validation & ID Computation
// ---------------------------------------------------------------------------

// ValidateAgent validates an Agent configuration and computes its content-addressed ID.
func ValidateAgent(agent AgentAgentBase) (*AgentAgent, error) {
	return callCFFI1[AgentAgentBase, AgentAgent](agent, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_validate_agent(a, b, c, d)
	})
}

// ValidateEnsemble validates an Ensemble configuration and computes its content-addressed ID.
func ValidateEnsemble(ensemble EnsembleEnsembleBase) (*EnsembleEnsemble, error) {
	return callCFFI1[EnsembleEnsembleBase, EnsembleEnsemble](ensemble, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_validate_ensemble(a, b, c, d)
	})
}

// PromptId computes a content-addressed ID for chat messages.
func PromptId(prompt []AgentCompletionsMessageMessage) (string, error) {
	result, err := callCFFI1Bytes(prompt, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_prompt_id(a, b, c, d)
	})
	if err != nil {
		return "", err
	}
	return string(result), nil
}

// VectorResponseId computes a content-addressed ID for a vector completion response option.
func VectorResponseId(response AgentCompletionsMessageRichContent) (string, error) {
	result, err := callCFFI1Bytes(response, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_vector_response_id(a, b, c, d)
	})
	if err != nil {
		return "", err
	}
	return string(result), nil
}

// ---------------------------------------------------------------------------
// Function Input Validation
// ---------------------------------------------------------------------------

// ValidateFunctionInput validates function input against its schema.
// Returns true if valid, false if invalid, nil if not applicable (inline function).
func ValidateFunctionInput(function FunctionsFunction, input FunctionsExpressionInputValue) (*bool, error) {
	jsonFn, err := json.Marshal(function)
	if err != nil {
		return nil, err
	}
	jsonIn, err := json.Marshal(input)
	if err != nil {
		return nil, err
	}

	var outPtr *C.uint8_t
	var outLen C.size_t

	rc := C.objectiveai_validate_function_input(
		(*C.uint8_t)(unsafe.Pointer(&jsonFn[0])), C.size_t(len(jsonFn)),
		(*C.uint8_t)(unsafe.Pointer(&jsonIn[0])), C.size_t(len(jsonIn)),
		&outPtr, &outLen,
	)
	C.objectiveai_free(outPtr, outLen)

	switch rc {
	case 1:
		v := true
		return &v, nil
	case 0:
		v := false
		return &v, nil
	case 2:
		return nil, nil
	default:
		return nil, errors.New(string(C.GoBytes(unsafe.Pointer(outPtr), C.int(outLen))))
	}
}

// ---------------------------------------------------------------------------
// Function Task Compilation
// ---------------------------------------------------------------------------

// CompileFunctionTasks compiles a Function's task expressions for a given input.
func CompileFunctionTasks(function FunctionsFunction, input FunctionsExpressionInputValue) ([]FunctionsCompiledTask, error) {
	out, err := callCFFI2[FunctionsFunction, FunctionsExpressionInputValue, []FunctionsCompiledTask](function, input, func(a *C.uint8_t, b C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_compile_function_tasks(a, b, c, d, e, f)
	})
	if err != nil {
		return nil, err
	}
	return *out, nil
}

// CompileFunctionOutputLength computes the expected output length for a vector Function.
func CompileFunctionOutputLength(function FunctionsFunction, input FunctionsExpressionInputValue) (*uint32, error) {
	return callCFFI2[FunctionsFunction, FunctionsExpressionInputValue, uint32](function, input, func(a *C.uint8_t, b C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_compile_function_output_length(a, b, c, d, e, f)
	})
}

// CompileFunctionInputSplit compiles the input_split expression.
func CompileFunctionInputSplit(function FunctionsFunction, input FunctionsExpressionInputValue) ([]FunctionsExpressionInputValue, error) {
	out, err := callCFFI2[FunctionsFunction, FunctionsExpressionInputValue, []FunctionsExpressionInputValue](function, input, func(a *C.uint8_t, b C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_compile_function_input_split(a, b, c, d, e, f)
	})
	if err != nil {
		return nil, err
	}
	if out == nil {
		return nil, nil
	}
	return *out, nil
}

// CompileFunctionInputMerge compiles the input_merge expression.
func CompileFunctionInputMerge(function FunctionsFunction, input []FunctionsExpressionInputValue) (*FunctionsExpressionInputValue, error) {
	return callCFFI2[FunctionsFunction, []FunctionsExpressionInputValue, FunctionsExpressionInputValue](function, input, func(a *C.uint8_t, b C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_compile_function_input_merge(a, b, c, d, e, f)
	})
}

// ---------------------------------------------------------------------------
// Vector/Scalar Field Validation
// ---------------------------------------------------------------------------

// CheckVectorFields validates vector function fields (output_length, input_split, input_merge).
func CheckVectorFields(fields FunctionsCheckVectorFieldsValidation) error {
	return callCFFI1Void(fields, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_check_vector_fields(a, b, c, d)
	})
}

// CheckScalarFields validates scalar function fields (input_schema only).
func CheckScalarFields(fields FunctionsCheckScalarFieldsValidation) error {
	return callCFFI1Void(fields, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_check_scalar_fields(a, b, c, d)
	})
}

// ---------------------------------------------------------------------------
// Alpha Function Validation
// ---------------------------------------------------------------------------

// AlphaCheckLeafScalarFunction validates a leaf scalar function (depth 0).
func AlphaCheckLeafScalarFunction(function FunctionsAlphaScalarRemoteFunction) error {
	return callCFFI1Void(function, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_alpha_check_leaf_scalar_function(a, b, c, d)
	})
}

// AlphaCheckBranchScalarFunction validates a branch scalar function (depth > 0).
func AlphaCheckBranchScalarFunction(function FunctionsAlphaScalarRemoteFunction, children map[string]FunctionsRemoteFunction) error {
	return callCFFI2Void(function, children, func(a *C.uint8_t, b C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_alpha_check_branch_scalar_function(a, b, c, d, e, f)
	})
}

// AlphaCheckLeafVectorFunction validates a leaf vector function (depth 0).
func AlphaCheckLeafVectorFunction(function FunctionsAlphaVectorRemoteFunction) error {
	return callCFFI1Void(function, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_alpha_check_leaf_vector_function(a, b, c, d)
	})
}

// AlphaCheckBranchVectorFunction validates a branch vector function (depth > 0).
func AlphaCheckBranchVectorFunction(function FunctionsAlphaVectorRemoteFunction, children map[string]FunctionsRemoteFunction) error {
	return callCFFI2Void(function, children, func(a *C.uint8_t, b C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_alpha_check_branch_vector_function(a, b, c, d, e, f)
	})
}

// ---------------------------------------------------------------------------
// Streaming Chunk Merging
// ---------------------------------------------------------------------------

// AgentCompletionChunkMerged merges two AgentCompletionChunks via push.
func AgentCompletionChunkMerged(a, b AgentCompletionsResponseStreamingAgentCompletionChunk) (*AgentCompletionsResponseStreamingAgentCompletionChunk, error) {
	return callCFFI2[AgentCompletionsResponseStreamingAgentCompletionChunk, AgentCompletionsResponseStreamingAgentCompletionChunk, AgentCompletionsResponseStreamingAgentCompletionChunk](a, b, func(a1 *C.uint8_t, b1 C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_agent_completion_chunk_merged(a1, b1, c, d, e, f)
	})
}

// VectorCompletionChunkMerged merges two VectorCompletionChunks via push.
func VectorCompletionChunkMerged(a, b VectorCompletionsResponseStreamingVectorCompletionChunk) (*VectorCompletionsResponseStreamingVectorCompletionChunk, error) {
	return callCFFI2[VectorCompletionsResponseStreamingVectorCompletionChunk, VectorCompletionsResponseStreamingVectorCompletionChunk, VectorCompletionsResponseStreamingVectorCompletionChunk](a, b, func(a1 *C.uint8_t, b1 C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_vector_completion_chunk_merged(a1, b1, c, d, e, f)
	})
}

// FunctionExecutionChunkMerged merges two FunctionExecutionChunks via push.
func FunctionExecutionChunkMerged(a, b FunctionsExecutionsResponseStreamingFunctionExecutionChunk) (*FunctionsExecutionsResponseStreamingFunctionExecutionChunk, error) {
	return callCFFI2[FunctionsExecutionsResponseStreamingFunctionExecutionChunk, FunctionsExecutionsResponseStreamingFunctionExecutionChunk, FunctionsExecutionsResponseStreamingFunctionExecutionChunk](a, b, func(a1 *C.uint8_t, b1 C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_function_execution_chunk_merged(a1, b1, c, d, e, f)
	})
}

// FunctionInventionChunkMerged merges two FunctionInventionChunks via push.
func FunctionInventionChunkMerged(a, b FunctionsInventionsResponseStreamingFunctionInventionChunk) (*FunctionsInventionsResponseStreamingFunctionInventionChunk, error) {
	return callCFFI2[FunctionsInventionsResponseStreamingFunctionInventionChunk, FunctionsInventionsResponseStreamingFunctionInventionChunk, FunctionsInventionsResponseStreamingFunctionInventionChunk](a, b, func(a1 *C.uint8_t, b1 C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_function_invention_chunk_merged(a1, b1, c, d, e, f)
	})
}

// FunctionInventionRecursiveChunkMerged merges two FunctionInventionRecursiveChunks via push.
func FunctionInventionRecursiveChunkMerged(a, b FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk) (*FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, error) {
	return callCFFI2[FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk](a, b, func(a1 *C.uint8_t, b1 C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_function_invention_recursive_chunk_merged(a1, b1, c, d, e, f)
	})
}

// FunctionProfileComputationChunkMerged merges two FunctionProfileComputationChunks via push.
func FunctionProfileComputationChunkMerged(a, b FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk) (*FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, error) {
	return callCFFI2[FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk](a, b, func(a1 *C.uint8_t, b1 C.size_t, c *C.uint8_t, d C.size_t, e **C.uint8_t, f *C.size_t) C.int32_t {
		return C.objectiveai_function_profile_computation_chunk_merged(a1, b1, c, d, e, f)
	})
}

// ---------------------------------------------------------------------------
// Streaming Chunk Normalization
// ---------------------------------------------------------------------------

// AgentCompletionChunkNormalized normalizes an AgentCompletionChunk by round-tripping through serde.
func AgentCompletionChunkNormalized(chunk AgentCompletionsResponseStreamingAgentCompletionChunk) (*AgentCompletionsResponseStreamingAgentCompletionChunk, error) {
	return callCFFI1[AgentCompletionsResponseStreamingAgentCompletionChunk, AgentCompletionsResponseStreamingAgentCompletionChunk](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_agent_completion_chunk_normalized(a, b, c, d)
	})
}

// VectorCompletionChunkNormalized normalizes a VectorCompletionChunk.
func VectorCompletionChunkNormalized(chunk VectorCompletionsResponseStreamingVectorCompletionChunk) (*VectorCompletionsResponseStreamingVectorCompletionChunk, error) {
	return callCFFI1[VectorCompletionsResponseStreamingVectorCompletionChunk, VectorCompletionsResponseStreamingVectorCompletionChunk](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_vector_completion_chunk_normalized(a, b, c, d)
	})
}

// FunctionExecutionChunkNormalized normalizes a FunctionExecutionChunk.
func FunctionExecutionChunkNormalized(chunk FunctionsExecutionsResponseStreamingFunctionExecutionChunk) (*FunctionsExecutionsResponseStreamingFunctionExecutionChunk, error) {
	return callCFFI1[FunctionsExecutionsResponseStreamingFunctionExecutionChunk, FunctionsExecutionsResponseStreamingFunctionExecutionChunk](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_execution_chunk_normalized(a, b, c, d)
	})
}

// FunctionInventionChunkNormalized normalizes a FunctionInventionChunk.
func FunctionInventionChunkNormalized(chunk FunctionsInventionsResponseStreamingFunctionInventionChunk) (*FunctionsInventionsResponseStreamingFunctionInventionChunk, error) {
	return callCFFI1[FunctionsInventionsResponseStreamingFunctionInventionChunk, FunctionsInventionsResponseStreamingFunctionInventionChunk](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_invention_chunk_normalized(a, b, c, d)
	})
}

// FunctionInventionRecursiveChunkNormalized normalizes a FunctionInventionRecursiveChunk.
func FunctionInventionRecursiveChunkNormalized(chunk FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk) (*FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, error) {
	return callCFFI1[FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_invention_recursive_chunk_normalized(a, b, c, d)
	})
}

// FunctionProfileComputationChunkNormalized normalizes a FunctionProfileComputationChunk.
func FunctionProfileComputationChunkNormalized(chunk FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk) (*FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, error) {
	return callCFFI1[FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_profile_computation_chunk_normalized(a, b, c, d)
	})
}

// ---------------------------------------------------------------------------
// Streaming Chunk to Unary Conversion
// ---------------------------------------------------------------------------

// AgentCompletionChunkToUnary converts an accumulated chunk to a unary AgentCompletion.
func AgentCompletionChunkToUnary(chunk AgentCompletionsResponseStreamingAgentCompletionChunk) (*AgentCompletionsResponseUnaryAgentCompletion, error) {
	return callCFFI1[AgentCompletionsResponseStreamingAgentCompletionChunk, AgentCompletionsResponseUnaryAgentCompletion](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_agent_completion_chunk_to_unary(a, b, c, d)
	})
}

// VectorCompletionChunkToUnary converts an accumulated chunk to a unary VectorCompletion.
func VectorCompletionChunkToUnary(chunk VectorCompletionsResponseStreamingVectorCompletionChunk) (*VectorCompletionsResponseUnaryVectorCompletion, error) {
	return callCFFI1[VectorCompletionsResponseStreamingVectorCompletionChunk, VectorCompletionsResponseUnaryVectorCompletion](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_vector_completion_chunk_to_unary(a, b, c, d)
	})
}

// FunctionExecutionChunkToUnary converts an accumulated chunk to a unary FunctionExecution.
func FunctionExecutionChunkToUnary(chunk FunctionsExecutionsResponseStreamingFunctionExecutionChunk) (*FunctionsExecutionsResponseUnaryFunctionExecution, error) {
	return callCFFI1[FunctionsExecutionsResponseStreamingFunctionExecutionChunk, FunctionsExecutionsResponseUnaryFunctionExecution](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_execution_chunk_to_unary(a, b, c, d)
	})
}

// FunctionInventionChunkToUnary converts an accumulated chunk to a unary FunctionInvention.
func FunctionInventionChunkToUnary(chunk FunctionsInventionsResponseStreamingFunctionInventionChunk) (*FunctionsInventionsResponseUnaryFunctionInvention, error) {
	return callCFFI1[FunctionsInventionsResponseStreamingFunctionInventionChunk, FunctionsInventionsResponseUnaryFunctionInvention](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_invention_chunk_to_unary(a, b, c, d)
	})
}

// FunctionInventionRecursiveChunkToUnary converts an accumulated chunk to a unary FunctionInventionRecursive.
func FunctionInventionRecursiveChunkToUnary(chunk FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk) (*FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursive, error) {
	return callCFFI1[FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursive](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_invention_recursive_chunk_to_unary(a, b, c, d)
	})
}

// FunctionProfileComputationChunkToUnary converts an accumulated chunk to a unary FunctionProfileComputation.
func FunctionProfileComputationChunkToUnary(chunk FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk) (*FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation, error) {
	return callCFFI1[FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation](chunk, func(a *C.uint8_t, b C.size_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_function_profile_computation_chunk_to_unary(a, b, c, d)
	})
}

// ---------------------------------------------------------------------------
// Generate Arbitrary Chunks
// ---------------------------------------------------------------------------

func generateChunk[Out any](hasSeed bool, seed int64, cfn func(C.int32_t, C.int64_t, **C.uint8_t, *C.size_t) C.int32_t) (*Out, error) {
	var hs C.int32_t
	if hasSeed {
		hs = 1
	}

	var outPtr *C.uint8_t
	var outLen C.size_t

	rc := cfn(hs, C.int64_t(seed), &outPtr, &outLen)

	result := C.GoBytes(unsafe.Pointer(outPtr), C.int(outLen))
	C.objectiveai_free(outPtr, outLen)

	if rc != 0 {
		return nil, errors.New(string(result))
	}

	var out Out
	if err := json.Unmarshal(result, &out); err != nil {
		return nil, err
	}
	return &out, nil
}

// GenerateAgentCompletionChunk generates a random AgentCompletionChunk from a seed.
func GenerateAgentCompletionChunk(hasSeed bool, seed int64) (*AgentCompletionsResponseStreamingAgentCompletionChunk, error) {
	return generateChunk[AgentCompletionsResponseStreamingAgentCompletionChunk](hasSeed, seed, func(a C.int32_t, b C.int64_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_generate_agent_completion_chunk(a, b, c, d)
	})
}

// GenerateVectorCompletionChunk generates a random VectorCompletionChunk from a seed.
func GenerateVectorCompletionChunk(hasSeed bool, seed int64) (*VectorCompletionsResponseStreamingVectorCompletionChunk, error) {
	return generateChunk[VectorCompletionsResponseStreamingVectorCompletionChunk](hasSeed, seed, func(a C.int32_t, b C.int64_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_generate_vector_completion_chunk(a, b, c, d)
	})
}

// GenerateFunctionExecutionChunk generates a random FunctionExecutionChunk from a seed.
func GenerateFunctionExecutionChunk(hasSeed bool, seed int64) (*FunctionsExecutionsResponseStreamingFunctionExecutionChunk, error) {
	return generateChunk[FunctionsExecutionsResponseStreamingFunctionExecutionChunk](hasSeed, seed, func(a C.int32_t, b C.int64_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_generate_function_execution_chunk(a, b, c, d)
	})
}

// GenerateFunctionInventionChunk generates a random FunctionInventionChunk from a seed.
func GenerateFunctionInventionChunk(hasSeed bool, seed int64) (*FunctionsInventionsResponseStreamingFunctionInventionChunk, error) {
	return generateChunk[FunctionsInventionsResponseStreamingFunctionInventionChunk](hasSeed, seed, func(a C.int32_t, b C.int64_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_generate_function_invention_chunk(a, b, c, d)
	})
}

// GenerateFunctionInventionRecursiveChunk generates a random FunctionInventionRecursiveChunk from a seed.
func GenerateFunctionInventionRecursiveChunk(hasSeed bool, seed int64) (*FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, error) {
	return generateChunk[FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk](hasSeed, seed, func(a C.int32_t, b C.int64_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_generate_function_invention_recursive_chunk(a, b, c, d)
	})
}

// GenerateFunctionProfileComputationChunk generates a random FunctionProfileComputationChunk from a seed.
func GenerateFunctionProfileComputationChunk(hasSeed bool, seed int64) (*FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, error) {
	return generateChunk[FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk](hasSeed, seed, func(a C.int32_t, b C.int64_t, c **C.uint8_t, d *C.size_t) C.int32_t {
		return C.objectiveai_generate_function_profile_computation_chunk(a, b, c, d)
	})
}
