//go:build !cgo

package objectiveai

import "errors"

var errNoCGo = errors.New("objectiveai: native FFI requires CGO_ENABLED=1 with the objectiveai-cffi static library")

func Free(_ []byte)                                                        {}
func ValidateAgent(_ AgentAgentBase) (*AgentAgent, error)                   { return nil, errNoCGo }
func ValidateEnsemble(_ EnsembleEnsembleBase) (*EnsembleEnsemble, error)    { return nil, errNoCGo }
func PromptId(_ []AgentCompletionsMessageMessage) (string, error)           { return "", errNoCGo }
func VectorResponseId(_ AgentCompletionsMessageRichContent) (string, error) { return "", errNoCGo }
func ValidateFunctionInput(_ FunctionsFunction, _ FunctionsExpressionInputValue) (*bool, error) {
	return nil, errNoCGo
}
func CompileFunctionTasks(_ FunctionsFunction, _ FunctionsExpressionInputValue) ([]FunctionsCompiledTask, error) {
	return nil, errNoCGo
}
func CompileFunctionOutputLength(_ FunctionsFunction, _ FunctionsExpressionInputValue) (*uint32, error) {
	return nil, errNoCGo
}
func CompileFunctionInputSplit(_ FunctionsFunction, _ FunctionsExpressionInputValue) ([]FunctionsExpressionInputValue, error) {
	return nil, errNoCGo
}
func CompileFunctionInputMerge(_ FunctionsFunction, _ []FunctionsExpressionInputValue) (*FunctionsExpressionInputValue, error) {
	return nil, errNoCGo
}
func CheckVectorFields(_ FunctionsCheckVectorFieldsValidation) error { return errNoCGo }
func CheckScalarFields(_ FunctionsCheckScalarFieldsValidation) error { return errNoCGo }
func AlphaCheckLeafScalarFunction(_ FunctionsAlphaScalarRemoteFunction) error {
	return errNoCGo
}
func AlphaCheckBranchScalarFunction(_ FunctionsAlphaScalarRemoteFunction, _ map[string]FunctionsRemoteFunction) error {
	return errNoCGo
}
func AlphaCheckLeafVectorFunction(_ FunctionsAlphaVectorRemoteFunction) error {
	return errNoCGo
}
func AlphaCheckBranchVectorFunction(_ FunctionsAlphaVectorRemoteFunction, _ map[string]FunctionsRemoteFunction) error {
	return errNoCGo
}
func AgentCompletionChunkMerged(_, _ AgentCompletionsResponseStreamingAgentCompletionChunk) (*AgentCompletionsResponseStreamingAgentCompletionChunk, error) {
	return nil, errNoCGo
}
func VectorCompletionChunkMerged(_, _ VectorCompletionsResponseStreamingVectorCompletionChunk) (*VectorCompletionsResponseStreamingVectorCompletionChunk, error) {
	return nil, errNoCGo
}
func FunctionExecutionChunkMerged(_, _ FunctionsExecutionsResponseStreamingFunctionExecutionChunk) (*FunctionsExecutionsResponseStreamingFunctionExecutionChunk, error) {
	return nil, errNoCGo
}
func FunctionInventionChunkMerged(_, _ FunctionsInventionsResponseStreamingFunctionInventionChunk) (*FunctionsInventionsResponseStreamingFunctionInventionChunk, error) {
	return nil, errNoCGo
}
func FunctionInventionRecursiveChunkMerged(_, _ FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk) (*FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, error) {
	return nil, errNoCGo
}
func FunctionProfileComputationChunkMerged(_, _ FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk) (*FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, error) {
	return nil, errNoCGo
}
func AgentCompletionChunkNormalized(_ AgentCompletionsResponseStreamingAgentCompletionChunk) (*AgentCompletionsResponseStreamingAgentCompletionChunk, error) {
	return nil, errNoCGo
}
func VectorCompletionChunkNormalized(_ VectorCompletionsResponseStreamingVectorCompletionChunk) (*VectorCompletionsResponseStreamingVectorCompletionChunk, error) {
	return nil, errNoCGo
}
func FunctionExecutionChunkNormalized(_ FunctionsExecutionsResponseStreamingFunctionExecutionChunk) (*FunctionsExecutionsResponseStreamingFunctionExecutionChunk, error) {
	return nil, errNoCGo
}
func FunctionInventionChunkNormalized(_ FunctionsInventionsResponseStreamingFunctionInventionChunk) (*FunctionsInventionsResponseStreamingFunctionInventionChunk, error) {
	return nil, errNoCGo
}
func FunctionInventionRecursiveChunkNormalized(_ FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk) (*FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, error) {
	return nil, errNoCGo
}
func FunctionProfileComputationChunkNormalized(_ FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk) (*FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, error) {
	return nil, errNoCGo
}
func AgentCompletionChunkToUnary(_ AgentCompletionsResponseStreamingAgentCompletionChunk) (*AgentCompletionsResponseUnaryAgentCompletion, error) {
	return nil, errNoCGo
}
func VectorCompletionChunkToUnary(_ VectorCompletionsResponseStreamingVectorCompletionChunk) (*VectorCompletionsResponseUnaryVectorCompletion, error) {
	return nil, errNoCGo
}
func FunctionExecutionChunkToUnary(_ FunctionsExecutionsResponseStreamingFunctionExecutionChunk) (*FunctionsExecutionsResponseUnaryFunctionExecution, error) {
	return nil, errNoCGo
}
func FunctionInventionChunkToUnary(_ FunctionsInventionsResponseStreamingFunctionInventionChunk) (*FunctionsInventionsResponseUnaryFunctionInvention, error) {
	return nil, errNoCGo
}
func FunctionInventionRecursiveChunkToUnary(_ FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk) (*FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursive, error) {
	return nil, errNoCGo
}
func FunctionProfileComputationChunkToUnary(_ FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk) (*FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation, error) {
	return nil, errNoCGo
}
func GenerateAgentCompletionChunk(_ bool, _ int64) (*AgentCompletionsResponseStreamingAgentCompletionChunk, error) {
	return nil, errNoCGo
}
func GenerateVectorCompletionChunk(_ bool, _ int64) (*VectorCompletionsResponseStreamingVectorCompletionChunk, error) {
	return nil, errNoCGo
}
func GenerateFunctionExecutionChunk(_ bool, _ int64) (*FunctionsExecutionsResponseStreamingFunctionExecutionChunk, error) {
	return nil, errNoCGo
}
func GenerateFunctionInventionChunk(_ bool, _ int64) (*FunctionsInventionsResponseStreamingFunctionInventionChunk, error) {
	return nil, errNoCGo
}
func GenerateFunctionInventionRecursiveChunk(_ bool, _ int64) (*FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk, error) {
	return nil, errNoCGo
}
func GenerateFunctionProfileComputationChunk(_ bool, _ int64) (*FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk, error) {
	return nil, errNoCGo
}
