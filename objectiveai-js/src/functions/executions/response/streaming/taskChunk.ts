import { z } from "zod";
import { FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunkSchema } from "./functionExecutionTaskChunk";
import { FunctionsExecutionsResponseStreamingVectorCompletionTaskChunkSchema } from "./vectorCompletionTaskChunk";

export const FunctionsExecutionsResponseStreamingTaskChunkSchema = z.union([FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunkSchema, FunctionsExecutionsResponseStreamingVectorCompletionTaskChunkSchema]).meta({ title: "functions.executions.response.streaming.TaskChunk" });
export type FunctionsExecutionsResponseStreamingTaskChunk = z.infer<typeof FunctionsExecutionsResponseStreamingTaskChunkSchema>;
