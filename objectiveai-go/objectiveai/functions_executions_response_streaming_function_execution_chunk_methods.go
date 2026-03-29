package objectiveai

// Push accumulates another FunctionExecutionChunk into this one.
func (v *FunctionsExecutionsResponseStreamingFunctionExecutionChunk) Push(other *FunctionsExecutionsResponseStreamingFunctionExecutionChunk) {
	// tasks: merge by index
	pushByIndex(&v.Tasks, other.Tasks,
		func(t *FunctionsExecutionsResponseStreamingTaskChunk) uint64 {
			if t.FunctionExecution != nil {
				return t.FunctionExecution.Index
			}
			if t.VectorCompletion != nil {
				return t.VectorCompletion.Index
			}
			return 0
		},
		func(a, b *FunctionsExecutionsResponseStreamingTaskChunk) { a.Push(b) },
	)

	// tasks_errors: lazy set true
	v.TasksErrors = pushLazySetTrue(v.TasksErrors, other.TasksErrors)

	// reasoning: delegate
	if v.Reasoning != nil && other.Reasoning != nil {
		v.Reasoning.Push(other.Reasoning)
	} else if other.Reasoning != nil {
		v.Reasoning = other.Reasoning
	}

	// output: replace
	v.Output = pushReplace(v.Output, other.Output)

	// retry_token: replace
	v.RetryToken = pushReplace(v.RetryToken, other.RetryToken)

	// error: replace
	v.Error = pushReplace(v.Error, other.Error)

	// usage: delegate
	if v.Usage != nil && other.Usage != nil {
		v.Usage.Push(other.Usage)
	} else if other.Usage != nil {
		v.Usage = other.Usage
	}

	// id, created, object, function, profile are immutable
}
