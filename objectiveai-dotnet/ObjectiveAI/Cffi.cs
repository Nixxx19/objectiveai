using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace ObjectiveAI;

/// <summary>
/// Native CFFI bindings to objectiveai-rs-cffi via P/Invoke.
/// Matches Go's cffi.go pattern: JSON in → native call → JSON out.
/// </summary>
public static class Cffi
{
    private const string Lib = "objectiveai_cffi";

    static Cffi()
    {
        // Register a custom resolver that looks in runtimes/{rid}/native/
        // relative to the ObjectiveAI assembly location. This handles the
        // project-reference case where NuGet's runtime resolution doesn't apply.
        NativeLibrary.SetDllImportResolver(typeof(Cffi).Assembly, (libraryName, assembly, searchPath) =>
        {
            if (libraryName != Lib)
                return nint.Zero;

            // Try default resolution first
            if (NativeLibrary.TryLoad(libraryName, assembly, searchPath, out var handle))
                return handle;

            // Try runtimes/{rid}/native/ relative to assembly
            var rid = RuntimeInformation.RuntimeIdentifier;
            var assemblyDir = Path.GetDirectoryName(assembly.Location) ?? ".";

            // Check alongside the assembly (e.g., bin/Release/net10.0/)
            var candidates = new[]
            {
                Path.Combine(assemblyDir, "runtimes", rid, "native", NativeLibFileName()),
                // Also check the ObjectiveAI project's runtimes/ folder (dev scenario)
                Path.Combine(assemblyDir, "..", "..", "..", "..", "ObjectiveAI", "runtimes", rid, "native", NativeLibFileName()),
            };

            foreach (var path in candidates)
            {
                if (File.Exists(path) && NativeLibrary.TryLoad(path, out handle))
                    return handle;
            }

            return nint.Zero;
        });
    }

    private static string NativeLibFileName()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows)) return "objectiveai_cffi.dll";
        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX)) return "libobjectiveai_cffi.dylib";
        return "libobjectiveai_cffi.so";
    }

    // -----------------------------------------------------------------------
    // Native extern declarations
    // -----------------------------------------------------------------------

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    private static extern nint objectiveai_allocate(nuint len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    private static extern void objectiveai_free(nint ptr, nuint len);

    // 1-input functions: (in, in_len, *out, *out_len) -> i32
    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_validate_agent")]
    private static extern int native_validate_agent(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_prompt_id")]
    private static extern int native_prompt_id(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_vector_response_id")]
    private static extern int native_vector_response_id(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_check_vector_fields")]
    private static extern int native_check_vector_fields(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_check_scalar_fields")]
    private static extern int native_check_scalar_fields(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_alpha_check_leaf_scalar_function")]
    private static extern int native_alpha_check_leaf_scalar_function(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_alpha_check_leaf_vector_function")]
    private static extern int native_alpha_check_leaf_vector_function(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_agent_completion_chunk_normalized")]
    private static extern int native_agent_completion_chunk_normalized(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_vector_completion_chunk_normalized")]
    private static extern int native_vector_completion_chunk_normalized(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_execution_chunk_normalized")]
    private static extern int native_function_execution_chunk_normalized(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_invention_chunk_normalized")]
    private static extern int native_function_invention_chunk_normalized(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_invention_recursive_chunk_normalized")]
    private static extern int native_function_invention_recursive_chunk_normalized(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_profile_computation_chunk_normalized")]
    private static extern int native_function_profile_computation_chunk_normalized(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_agent_completion_chunk_to_unary")]
    private static extern int native_agent_completion_chunk_to_unary(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_vector_completion_chunk_to_unary")]
    private static extern int native_vector_completion_chunk_to_unary(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_execution_chunk_to_unary")]
    private static extern int native_function_execution_chunk_to_unary(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_invention_chunk_to_unary")]
    private static extern int native_function_invention_chunk_to_unary(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_invention_recursive_chunk_to_unary")]
    private static extern int native_function_invention_recursive_chunk_to_unary(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_profile_computation_chunk_to_unary")]
    private static extern int native_function_profile_computation_chunk_to_unary(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_normalize_agent_completion_for_tests")]
    private static extern int native_normalize_agent_completion_for_tests(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_normalize_vector_completion_for_tests")]
    private static extern int native_normalize_vector_completion_for_tests(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_normalize_function_execution_for_tests")]
    private static extern int native_normalize_function_execution_for_tests(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_normalize_function_invention_for_tests")]
    private static extern int native_normalize_function_invention_for_tests(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_normalize_function_invention_recursive_for_tests")]
    private static extern int native_normalize_function_invention_recursive_for_tests(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_normalize_function_profile_computation_for_tests")]
    private static extern int native_normalize_function_profile_computation_for_tests(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    // 2-input functions: (in1, in1_len, in2, in2_len, *out, *out_len) -> i32
    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_validate_swarm")]
    private static extern int native_validate_swarm(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_validate_function_input")]
    private static extern int native_validate_function_input(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_compile_function_tasks")]
    private static extern int native_compile_function_tasks(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_compile_function_output_length")]
    private static extern int native_compile_function_output_length(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_compile_function_input_split")]
    private static extern int native_compile_function_input_split(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_compile_function_input_merge")]
    private static extern int native_compile_function_input_merge(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_alpha_check_branch_scalar_function")]
    private static extern int native_alpha_check_branch_scalar_function(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_alpha_check_branch_vector_function")]
    private static extern int native_alpha_check_branch_vector_function(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_agent_completion_chunk_merged")]
    private static extern int native_agent_completion_chunk_merged(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_vector_completion_chunk_merged")]
    private static extern int native_vector_completion_chunk_merged(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_execution_chunk_merged")]
    private static extern int native_function_execution_chunk_merged(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_invention_chunk_merged")]
    private static extern int native_function_invention_chunk_merged(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_invention_recursive_chunk_merged")]
    private static extern int native_function_invention_recursive_chunk_merged(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_function_profile_computation_chunk_merged")]
    private static extern int native_function_profile_computation_chunk_merged(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    // Seed-based functions: (has_seed, seed, *out, *out_len) -> i32
    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_generate_agent_completion_chunk")]
    private static extern int native_generate_agent_completion_chunk(int has_seed, long seed, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_generate_vector_completion_chunk")]
    private static extern int native_generate_vector_completion_chunk(int has_seed, long seed, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_generate_function_execution_chunk")]
    private static extern int native_generate_function_execution_chunk(int has_seed, long seed, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_generate_function_invention_chunk")]
    private static extern int native_generate_function_invention_chunk(int has_seed, long seed, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_generate_function_invention_recursive_chunk")]
    private static extern int native_generate_function_invention_recursive_chunk(int has_seed, long seed, out nint out_ptr, out nuint out_len);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "objectiveai_generate_function_profile_computation_chunk")]
    private static extern int native_generate_function_profile_computation_chunk(int has_seed, long seed, out nint out_ptr, out nuint out_len);

    // -----------------------------------------------------------------------
    // Low-level call helpers
    // -----------------------------------------------------------------------

    private static readonly object Lock = new();
    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull,
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
    };

    private static byte[] ReadOutput(nint outPtr, nuint outLen)
    {
        if (outLen == 0) return [];
        var buf = new byte[(int)outLen];
        Marshal.Copy(outPtr, buf, 0, (int)outLen);
        objectiveai_free(outPtr, outLen);
        return buf;
    }

    private delegate int Native1(nint json_in, nuint in_len, out nint out_ptr, out nuint out_len);

    private static (byte[] data, int rc) Call1(Native1 fn, byte[] jsonIn)
    {
        lock (Lock)
        {
            var pinned = GCHandle.Alloc(jsonIn, GCHandleType.Pinned);
            try
            {
                var rc = fn(pinned.AddrOfPinnedObject(), (nuint)jsonIn.Length, out var outPtr, out var outLen);
                return (ReadOutput(outPtr, outLen), rc);
            }
            finally { pinned.Free(); }
        }
    }

    private delegate int Native2(nint in1, nuint in1_len, nint in2, nuint in2_len, out nint out_ptr, out nuint out_len);

    private static (byte[] data, int rc) Call2(Native2 fn, byte[] jsonIn1, byte[] jsonIn2)
    {
        lock (Lock)
        {
            var pin1 = GCHandle.Alloc(jsonIn1, GCHandleType.Pinned);
            var pin2 = GCHandle.Alloc(jsonIn2, GCHandleType.Pinned);
            try
            {
                var rc = fn(
                    pin1.AddrOfPinnedObject(), (nuint)jsonIn1.Length,
                    pin2.AddrOfPinnedObject(), (nuint)jsonIn2.Length,
                    out var outPtr, out var outLen);
                return (ReadOutput(outPtr, outLen), rc);
            }
            finally { pin1.Free(); pin2.Free(); }
        }
    }

    private delegate int NativeSeed(int has_seed, long seed, out nint out_ptr, out nuint out_len);

    private static byte[] CallSeed(NativeSeed fn, long seed)
    {
        lock (Lock)
        {
            var rc = fn(1, seed, out var outPtr, out var outLen);
            var data = ReadOutput(outPtr, outLen);
            if (rc != 0)
                throw new InvalidOperationException(Encoding.UTF8.GetString(data));
            return data;
        }
    }

    // -----------------------------------------------------------------------
    // Typed helper wrappers
    // -----------------------------------------------------------------------

    private static TOut Cffi1<TIn, TOut>(Native1 fn, TIn input)
    {
        var jsonIn = JsonSerializer.SerializeToUtf8Bytes(input, JsonOpts);
        var (data, rc) = Call1(fn, jsonIn);
        if (rc != 0)
            throw new InvalidOperationException(Encoding.UTF8.GetString(data));
        return JsonSerializer.Deserialize<TOut>(data, JsonOpts)!;
    }

    private static string Cffi1String<TIn>(Native1 fn, TIn input)
    {
        var jsonIn = JsonSerializer.SerializeToUtf8Bytes(input, JsonOpts);
        var (data, rc) = Call1(fn, jsonIn);
        if (rc != 0)
            throw new InvalidOperationException(Encoding.UTF8.GetString(data));
        return Encoding.UTF8.GetString(data);
    }

    private static void Cffi1Void<TIn>(Native1 fn, TIn input)
    {
        var jsonIn = JsonSerializer.SerializeToUtf8Bytes(input, JsonOpts);
        var (data, rc) = Call1(fn, jsonIn);
        if (rc != 0)
            throw new InvalidOperationException(Encoding.UTF8.GetString(data));
    }

    private static TOut Cffi2<TIn1, TIn2, TOut>(Native2 fn, TIn1 input1, TIn2 input2)
    {
        var jsonIn1 = JsonSerializer.SerializeToUtf8Bytes(input1, JsonOpts);
        var jsonIn2 = JsonSerializer.SerializeToUtf8Bytes(input2, JsonOpts);
        var (data, rc) = Call2(fn, jsonIn1, jsonIn2);
        if (rc != 0)
            throw new InvalidOperationException(Encoding.UTF8.GetString(data));
        return JsonSerializer.Deserialize<TOut>(data, JsonOpts)!;
    }

    private static void Cffi2Void<TIn1, TIn2>(Native2 fn, TIn1 input1, TIn2 input2)
    {
        var jsonIn1 = JsonSerializer.SerializeToUtf8Bytes(input1, JsonOpts);
        var jsonIn2 = JsonSerializer.SerializeToUtf8Bytes(input2, JsonOpts);
        var (data, rc) = Call2(fn, jsonIn1, jsonIn2);
        if (rc != 0)
            throw new InvalidOperationException(Encoding.UTF8.GetString(data));
    }

    private static TOut CffiGenerate<TOut>(NativeSeed fn, long seed)
    {
        var data = CallSeed(fn, seed);
        return JsonSerializer.Deserialize<TOut>(data, JsonOpts)!;
    }

    // -----------------------------------------------------------------------
    // Validation & ID Computation
    // -----------------------------------------------------------------------

    public static Agent.Agent ValidateAgent(Agent.AgentBase agent)
        => Cffi1<Agent.AgentBase, Agent.Agent>(native_validate_agent, agent);

    public static Swarm.Swarm ValidateSwarm(Swarm.SwarmBase swarm, Dictionary<string, Agent.RemoteAgentBaseWithFallbacks>? remoteAgents)
        => Cffi2<Swarm.SwarmBase, Dictionary<string, Agent.RemoteAgentBaseWithFallbacks>?, Swarm.Swarm>(native_validate_swarm, swarm, remoteAgents);

    public static string PromptId(List<Agent.Completions.Message.Message> prompt)
        => Cffi1String(native_prompt_id, prompt);

    public static string VectorResponseId(Agent.Completions.Message.RichContent response)
        => Cffi1String(native_vector_response_id, response);

    // -----------------------------------------------------------------------
    // Function Input Validation
    // -----------------------------------------------------------------------

    public static int ValidateFunctionInput(Functions.Function function, Functions.Expression.InputValue input)
    {
        var jsonIn1 = JsonSerializer.SerializeToUtf8Bytes(function, JsonOpts);
        var jsonIn2 = JsonSerializer.SerializeToUtf8Bytes(input, JsonOpts);
        var (_, rc) = Call2(native_validate_function_input, jsonIn1, jsonIn2);
        return rc; // 1 = valid, 0 = invalid, 2 = not applicable
    }

    // -----------------------------------------------------------------------
    // Function Task Compilation
    // -----------------------------------------------------------------------

    public static List<Functions.CompiledTask> CompileFunctionTasks(Functions.Function function, Functions.Expression.InputValue input)
        => Cffi2<Functions.Function, Functions.Expression.InputValue, List<Functions.CompiledTask>>(native_compile_function_tasks, function, input);

    public static uint CompileFunctionOutputLength(Functions.Function function, Functions.Expression.InputValue input)
        => Cffi2<Functions.Function, Functions.Expression.InputValue, uint>(native_compile_function_output_length, function, input);

    public static List<Functions.Expression.InputValue>? CompileFunctionInputSplit(Functions.Function function, Functions.Expression.InputValue input)
        => Cffi2<Functions.Function, Functions.Expression.InputValue, List<Functions.Expression.InputValue>?>(native_compile_function_input_split, function, input);

    public static Functions.Expression.InputValue CompileFunctionInputMerge(Functions.Function function, List<Functions.Expression.InputValue> input)
        => Cffi2<Functions.Function, List<Functions.Expression.InputValue>, Functions.Expression.InputValue>(native_compile_function_input_merge, function, input);

    // -----------------------------------------------------------------------
    // Vector/Scalar Field Validation
    // -----------------------------------------------------------------------

    public static void CheckVectorFields(Functions.Check.VectorFieldsValidation fields)
        => Cffi1Void(native_check_vector_fields, fields);

    public static void CheckScalarFields(Functions.Check.ScalarFieldsValidation fields)
        => Cffi1Void(native_check_scalar_fields, fields);

    // -----------------------------------------------------------------------
    // Alpha Function Validation
    // -----------------------------------------------------------------------

    public static void AlphaCheckLeafScalarFunction(Functions.AlphaScalar.RemoteFunction function)
        => Cffi1Void(native_alpha_check_leaf_scalar_function, function);

    public static void AlphaCheckBranchScalarFunction(Functions.AlphaScalar.RemoteFunction function, Dictionary<string, Functions.FullRemoteFunction> children)
        => Cffi2Void(native_alpha_check_branch_scalar_function, function, children);

    public static void AlphaCheckLeafVectorFunction(Functions.AlphaVector.RemoteFunction function)
        => Cffi1Void(native_alpha_check_leaf_vector_function, function);

    public static void AlphaCheckBranchVectorFunction(Functions.AlphaVector.RemoteFunction function, Dictionary<string, Functions.FullRemoteFunction> children)
        => Cffi2Void(native_alpha_check_branch_vector_function, function, children);

    // -----------------------------------------------------------------------
    // Streaming Chunk Merging
    // -----------------------------------------------------------------------

    public static Agent.Completions.Response.Streaming.AgentCompletionChunk AgentCompletionChunkMerged(
        Agent.Completions.Response.Streaming.AgentCompletionChunk a,
        Agent.Completions.Response.Streaming.AgentCompletionChunk b)
        => Cffi2<Agent.Completions.Response.Streaming.AgentCompletionChunk, Agent.Completions.Response.Streaming.AgentCompletionChunk, Agent.Completions.Response.Streaming.AgentCompletionChunk>(native_agent_completion_chunk_merged, a, b);

    public static Vector.Completions.Response.Streaming.VectorCompletionChunk VectorCompletionChunkMerged(
        Vector.Completions.Response.Streaming.VectorCompletionChunk a,
        Vector.Completions.Response.Streaming.VectorCompletionChunk b)
        => Cffi2<Vector.Completions.Response.Streaming.VectorCompletionChunk, Vector.Completions.Response.Streaming.VectorCompletionChunk, Vector.Completions.Response.Streaming.VectorCompletionChunk>(native_vector_completion_chunk_merged, a, b);

    public static Functions.Executions.Response.Streaming.FunctionExecutionChunk FunctionExecutionChunkMerged(
        Functions.Executions.Response.Streaming.FunctionExecutionChunk a,
        Functions.Executions.Response.Streaming.FunctionExecutionChunk b)
        => Cffi2<Functions.Executions.Response.Streaming.FunctionExecutionChunk, Functions.Executions.Response.Streaming.FunctionExecutionChunk, Functions.Executions.Response.Streaming.FunctionExecutionChunk>(native_function_execution_chunk_merged, a, b);

    public static Functions.Inventions.Response.Streaming.FunctionInventionChunk FunctionInventionChunkMerged(
        Functions.Inventions.Response.Streaming.FunctionInventionChunk a,
        Functions.Inventions.Response.Streaming.FunctionInventionChunk b)
        => Cffi2<Functions.Inventions.Response.Streaming.FunctionInventionChunk, Functions.Inventions.Response.Streaming.FunctionInventionChunk, Functions.Inventions.Response.Streaming.FunctionInventionChunk>(native_function_invention_chunk_merged, a, b);

    public static Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk FunctionInventionRecursiveChunkMerged(
        Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk a,
        Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk b)
        => Cffi2<Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk, Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk, Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk>(native_function_invention_recursive_chunk_merged, a, b);

    public static Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk FunctionProfileComputationChunkMerged(
        Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk a,
        Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk b)
        => Cffi2<Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk, Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk, Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk>(native_function_profile_computation_chunk_merged, a, b);

    // -----------------------------------------------------------------------
    // Streaming Chunk Normalization
    // -----------------------------------------------------------------------

    public static Agent.Completions.Response.Streaming.AgentCompletionChunk AgentCompletionChunkNormalized(Agent.Completions.Response.Streaming.AgentCompletionChunk chunk)
        => Cffi1<Agent.Completions.Response.Streaming.AgentCompletionChunk, Agent.Completions.Response.Streaming.AgentCompletionChunk>(native_agent_completion_chunk_normalized, chunk);

    public static Vector.Completions.Response.Streaming.VectorCompletionChunk VectorCompletionChunkNormalized(Vector.Completions.Response.Streaming.VectorCompletionChunk chunk)
        => Cffi1<Vector.Completions.Response.Streaming.VectorCompletionChunk, Vector.Completions.Response.Streaming.VectorCompletionChunk>(native_vector_completion_chunk_normalized, chunk);

    public static Functions.Executions.Response.Streaming.FunctionExecutionChunk FunctionExecutionChunkNormalized(Functions.Executions.Response.Streaming.FunctionExecutionChunk chunk)
        => Cffi1<Functions.Executions.Response.Streaming.FunctionExecutionChunk, Functions.Executions.Response.Streaming.FunctionExecutionChunk>(native_function_execution_chunk_normalized, chunk);

    public static Functions.Inventions.Response.Streaming.FunctionInventionChunk FunctionInventionChunkNormalized(Functions.Inventions.Response.Streaming.FunctionInventionChunk chunk)
        => Cffi1<Functions.Inventions.Response.Streaming.FunctionInventionChunk, Functions.Inventions.Response.Streaming.FunctionInventionChunk>(native_function_invention_chunk_normalized, chunk);

    public static Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk FunctionInventionRecursiveChunkNormalized(Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk chunk)
        => Cffi1<Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk, Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk>(native_function_invention_recursive_chunk_normalized, chunk);

    public static Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk FunctionProfileComputationChunkNormalized(Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk chunk)
        => Cffi1<Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk, Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk>(native_function_profile_computation_chunk_normalized, chunk);

    // -----------------------------------------------------------------------
    // Streaming Chunk to Unary Conversion
    // -----------------------------------------------------------------------

    public static Agent.Completions.Response.Unary.AgentCompletion AgentCompletionChunkToUnary(Agent.Completions.Response.Streaming.AgentCompletionChunk chunk)
        => Cffi1<Agent.Completions.Response.Streaming.AgentCompletionChunk, Agent.Completions.Response.Unary.AgentCompletion>(native_agent_completion_chunk_to_unary, chunk);

    public static Vector.Completions.Response.Unary.VectorCompletion VectorCompletionChunkToUnary(Vector.Completions.Response.Streaming.VectorCompletionChunk chunk)
        => Cffi1<Vector.Completions.Response.Streaming.VectorCompletionChunk, Vector.Completions.Response.Unary.VectorCompletion>(native_vector_completion_chunk_to_unary, chunk);

    public static Functions.Executions.Response.Unary.FunctionExecution FunctionExecutionChunkToUnary(Functions.Executions.Response.Streaming.FunctionExecutionChunk chunk)
        => Cffi1<Functions.Executions.Response.Streaming.FunctionExecutionChunk, Functions.Executions.Response.Unary.FunctionExecution>(native_function_execution_chunk_to_unary, chunk);

    public static Functions.Inventions.Response.Unary.FunctionInvention FunctionInventionChunkToUnary(Functions.Inventions.Response.Streaming.FunctionInventionChunk chunk)
        => Cffi1<Functions.Inventions.Response.Streaming.FunctionInventionChunk, Functions.Inventions.Response.Unary.FunctionInvention>(native_function_invention_chunk_to_unary, chunk);

    public static Functions.Inventions.Recursive.Response.Unary.FunctionInventionRecursive FunctionInventionRecursiveChunkToUnary(Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk chunk)
        => Cffi1<Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk, Functions.Inventions.Recursive.Response.Unary.FunctionInventionRecursive>(native_function_invention_recursive_chunk_to_unary, chunk);

    public static Functions.Profiles.Computations.Response.Unary.FunctionProfileComputation FunctionProfileComputationChunkToUnary(Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk chunk)
        => Cffi1<Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk, Functions.Profiles.Computations.Response.Unary.FunctionProfileComputation>(native_function_profile_computation_chunk_to_unary, chunk);

    // -----------------------------------------------------------------------
    // Normalize Unary Responses (for tests)
    // -----------------------------------------------------------------------

    public static Agent.Completions.Response.Unary.AgentCompletion NormalizeAgentCompletionForTests(Agent.Completions.Response.Unary.AgentCompletion v)
        => Cffi1<Agent.Completions.Response.Unary.AgentCompletion, Agent.Completions.Response.Unary.AgentCompletion>(native_normalize_agent_completion_for_tests, v);

    public static Vector.Completions.Response.Unary.VectorCompletion NormalizeVectorCompletionForTests(Vector.Completions.Response.Unary.VectorCompletion v)
        => Cffi1<Vector.Completions.Response.Unary.VectorCompletion, Vector.Completions.Response.Unary.VectorCompletion>(native_normalize_vector_completion_for_tests, v);

    public static Functions.Executions.Response.Unary.FunctionExecution NormalizeFunctionExecutionForTests(Functions.Executions.Response.Unary.FunctionExecution v)
        => Cffi1<Functions.Executions.Response.Unary.FunctionExecution, Functions.Executions.Response.Unary.FunctionExecution>(native_normalize_function_execution_for_tests, v);

    public static Functions.Inventions.Response.Unary.FunctionInvention NormalizeFunctionInventionForTests(Functions.Inventions.Response.Unary.FunctionInvention v)
        => Cffi1<Functions.Inventions.Response.Unary.FunctionInvention, Functions.Inventions.Response.Unary.FunctionInvention>(native_normalize_function_invention_for_tests, v);

    public static Functions.Inventions.Recursive.Response.Unary.FunctionInventionRecursive NormalizeFunctionInventionRecursiveForTests(Functions.Inventions.Recursive.Response.Unary.FunctionInventionRecursive v)
        => Cffi1<Functions.Inventions.Recursive.Response.Unary.FunctionInventionRecursive, Functions.Inventions.Recursive.Response.Unary.FunctionInventionRecursive>(native_normalize_function_invention_recursive_for_tests, v);

    public static Functions.Profiles.Computations.Response.Unary.FunctionProfileComputation NormalizeFunctionProfileComputationForTests(Functions.Profiles.Computations.Response.Unary.FunctionProfileComputation v)
        => Cffi1<Functions.Profiles.Computations.Response.Unary.FunctionProfileComputation, Functions.Profiles.Computations.Response.Unary.FunctionProfileComputation>(native_normalize_function_profile_computation_for_tests, v);

    // -----------------------------------------------------------------------
    // Generate Arbitrary Chunks
    // -----------------------------------------------------------------------

    public static Agent.Completions.Response.Streaming.AgentCompletionChunk GenerateAgentCompletionChunk(long seed)
        => CffiGenerate<Agent.Completions.Response.Streaming.AgentCompletionChunk>(native_generate_agent_completion_chunk, seed);

    public static Vector.Completions.Response.Streaming.VectorCompletionChunk GenerateVectorCompletionChunk(long seed)
        => CffiGenerate<Vector.Completions.Response.Streaming.VectorCompletionChunk>(native_generate_vector_completion_chunk, seed);

    public static Functions.Executions.Response.Streaming.FunctionExecutionChunk GenerateFunctionExecutionChunk(long seed)
        => CffiGenerate<Functions.Executions.Response.Streaming.FunctionExecutionChunk>(native_generate_function_execution_chunk, seed);

    public static Functions.Inventions.Response.Streaming.FunctionInventionChunk GenerateFunctionInventionChunk(long seed)
        => CffiGenerate<Functions.Inventions.Response.Streaming.FunctionInventionChunk>(native_generate_function_invention_chunk, seed);

    public static Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk GenerateFunctionInventionRecursiveChunk(long seed)
        => CffiGenerate<Functions.Inventions.Recursive.Response.Streaming.FunctionInventionRecursiveChunk>(native_generate_function_invention_recursive_chunk, seed);

    public static Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk GenerateFunctionProfileComputationChunk(long seed)
        => CffiGenerate<Functions.Profiles.Computations.Response.Streaming.FunctionProfileComputationChunk>(native_generate_function_profile_computation_chunk, seed);

    // -----------------------------------------------------------------------
    // Memory Management
    // -----------------------------------------------------------------------

    public static void Allocate(nuint len) => objectiveai_allocate(len);
    public static void Free(nint ptr, nuint len) => objectiveai_free(ptr, len);
}
