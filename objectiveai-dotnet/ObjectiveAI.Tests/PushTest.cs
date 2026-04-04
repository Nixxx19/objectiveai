/*
 * Push fuzz tests: verify C# native Push() matches CFFI reference merge.
 *
 * Matches Go's *_push_test.go, JS's *ChunkMerged.test.ts,
 * and Python's test_*_chunk_push.py.
 *
 * For each of 6 chunk types, runs 20 streams × 20 chunks = 400 comparisons.
 * Flow: Generate(CFFI) → Push(C#) vs Merge(CFFI) → Compare(rounded).
 */

using ObjectiveAI.Agent.Completions.Response.Streaming;
using VectorStreaming = ObjectiveAI.Vector.Completions.Response.Streaming;
using FuncExecStreaming = ObjectiveAI.Functions.Executions.Response.Streaming;
using FuncInvStreaming = ObjectiveAI.Functions.Inventions.Response.Streaming;
using FuncInvRecStreaming = ObjectiveAI.Functions.Inventions.Recursive.Response.Streaming;
using FuncProfStreaming = ObjectiveAI.Functions.Profiles.Computations.Response.Streaming;
using static ObjectiveAI.Tests.PushTestUtils;

namespace ObjectiveAI.Tests;

public class PushTest
{
    private const int NumStreams = 20;
    private const int ChunksPerStream = 20;

    [Theory]
    [MemberData(nameof(StreamSeeds))]
    public void AgentCompletionChunkPush(int stream)
    {
        var seed = (long)stream * 1000;
        var csAcc = Cffi.GenerateAgentCompletionChunk(seed);
        var cffiAcc = DeepCopy(csAcc);
        seed++;

        for (int j = 0; j < ChunksPerStream; j++)
        {
            var chunk = Cffi.GenerateAgentCompletionChunk(seed++);
            csAcc.Push(chunk);
            cffiAcc = Cffi.AgentCompletionChunkMerged(cffiAcc, chunk);
            AssertRoundedEqual($"stream {stream} chunk {j}", ToMap(csAcc), ToMap(cffiAcc));
        }
    }

    [Theory]
    [MemberData(nameof(StreamSeeds))]
    public void VectorCompletionChunkPush(int stream)
    {
        var seed = (long)stream * 1000;
        var csAcc = Cffi.GenerateVectorCompletionChunk(seed);
        var cffiAcc = DeepCopy(csAcc);
        seed++;

        for (int j = 0; j < ChunksPerStream; j++)
        {
            var chunk = Cffi.GenerateVectorCompletionChunk(seed++);
            csAcc.Push(chunk);
            cffiAcc = Cffi.VectorCompletionChunkMerged(cffiAcc, chunk);
            AssertRoundedEqual($"stream {stream} chunk {j}", ToMap(csAcc), ToMap(cffiAcc));
        }
    }

    [Theory]
    [MemberData(nameof(StreamSeeds))]
    public void FunctionExecutionChunkPush(int stream)
    {
        var seed = (long)stream * 1000;
        var csAcc = Cffi.GenerateFunctionExecutionChunk(seed);
        var cffiAcc = DeepCopy(csAcc);
        seed++;

        for (int j = 0; j < ChunksPerStream; j++)
        {
            var chunk = Cffi.GenerateFunctionExecutionChunk(seed++);
            csAcc.Push(chunk);
            cffiAcc = Cffi.FunctionExecutionChunkMerged(cffiAcc, chunk);
            AssertRoundedEqual($"stream {stream} chunk {j}", ToMap(csAcc), ToMap(cffiAcc));
        }
    }

    [Theory]
    [MemberData(nameof(StreamSeeds))]
    public void FunctionInventionChunkPush(int stream)
    {
        var seed = (long)stream * 1000;
        var csAcc = Cffi.GenerateFunctionInventionChunk(seed);
        var cffiAcc = DeepCopy(csAcc);
        seed++;

        for (int j = 0; j < ChunksPerStream; j++)
        {
            var chunk = Cffi.GenerateFunctionInventionChunk(seed++);
            csAcc.Push(chunk);
            cffiAcc = Cffi.FunctionInventionChunkMerged(cffiAcc, chunk);
            AssertRoundedEqual($"stream {stream} chunk {j}", ToMap(csAcc), ToMap(cffiAcc));
        }
    }

    [Theory]
    [MemberData(nameof(StreamSeeds))]
    public void FunctionInventionRecursiveChunkPush(int stream)
    {
        var seed = (long)stream * 1000;
        var csAcc = Cffi.GenerateFunctionInventionRecursiveChunk(seed);
        var cffiAcc = DeepCopy(csAcc);
        seed++;

        for (int j = 0; j < ChunksPerStream; j++)
        {
            var chunk = Cffi.GenerateFunctionInventionRecursiveChunk(seed++);
            csAcc.Push(chunk);
            cffiAcc = Cffi.FunctionInventionRecursiveChunkMerged(cffiAcc, chunk);
            AssertRoundedEqual($"stream {stream} chunk {j}", ToMap(csAcc), ToMap(cffiAcc));
        }
    }

    [Theory]
    [MemberData(nameof(StreamSeeds))]
    public void FunctionProfileComputationChunkPush(int stream)
    {
        var seed = (long)stream * 1000;
        var csAcc = Cffi.GenerateFunctionProfileComputationChunk(seed);
        var cffiAcc = DeepCopy(csAcc);
        seed++;

        for (int j = 0; j < ChunksPerStream; j++)
        {
            var chunk = Cffi.GenerateFunctionProfileComputationChunk(seed++);
            csAcc.Push(chunk);
            cffiAcc = Cffi.FunctionProfileComputationChunkMerged(cffiAcc, chunk);
            AssertRoundedEqual($"stream {stream} chunk {j}", ToMap(csAcc), ToMap(cffiAcc));
        }
    }

    public static IEnumerable<object[]> StreamSeeds =>
        Enumerable.Range(0, NumStreams).Select(i => new object[] { i });
}
