import { z } from "zod";
import { AgentCompletionsRequestAgentSchema } from "../../../agent/completions/request/agent";
import { AgentCompletionsRequestProviderSchema } from "../../../agent/completions/request/provider";
import { FunctionsInventionsStateParamsStateSchema } from "../state/paramsState";
import { FunctionsRemoteSchema } from "../../remote";

export const FunctionsInventionsRequestFunctionInventionCreateParamsSchema = z.object({
  remote: FunctionsRemoteSchema.nullable().optional(),
  overwrite: z.boolean().nullable().optional(),
  state: FunctionsInventionsStateParamsStateSchema,
  provider: AgentCompletionsRequestProviderSchema.nullable().optional(),
  agent: AgentCompletionsRequestAgentSchema,
  agents: z.array(AgentCompletionsRequestAgentSchema).nullable().optional(),
  seed: z.number().int().meta({ format: "int64" }).nullable().optional(),
  stream: z.boolean().nullable().optional(),
  max_step_retries: z.number().int().min(0).meta({ format: "uint32" }).nullable().describe("Maximum number of retries per invention step.\nEach step is one agent completion (which itself may loop internally\nvia tool calls). If the step's validation still fails after the\nagent loop ends, the step is retried up to this many times.\nDefaults to 3 if not specified.").optional(),
  mcp_server_authorization: z.record(z.string(), z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional(),
}).meta({ title: "functions.inventions.request.FunctionInventionCreateParams" });
export type FunctionsInventionsRequestFunctionInventionCreateParams = z.infer<typeof FunctionsInventionsRequestFunctionInventionCreateParamsSchema>;
