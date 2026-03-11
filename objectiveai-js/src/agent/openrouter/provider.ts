import { z } from "zod";
import { AgentOpenrouterProviderQuantizationSchema } from "./providerQuantization";

export const AgentOpenrouterProviderSchema = z.object({
  allow_fallbacks: z.boolean().nullable().describe("Whether to allow fallback to other providers if preferred ones fail.\nDefaults to `true`.").optional(),
  require_parameters: z.boolean().nullable().describe("Whether to require that the provider supports all request parameters.\nDefaults to `false`.").optional(),
  order: z.array(z.string()).nullable().describe("Preferred provider order. Earlier providers are tried first.").optional(),
  only: z.array(z.string()).nullable().describe("Exclusive list of allowed providers. If set, only these providers are used.").optional(),
  ignore: z.array(z.string()).nullable().describe("Providers to exclude from routing.").optional(),
  quantizations: z.array(AgentOpenrouterProviderQuantizationSchema).nullable().describe("Allowed model quantization levels.").optional(),
}).describe("Provider routing preferences.\n\nControls which providers are used and in what order when routing\nrequests to upstream model hosts.").meta({ title: "agent.openrouter.Provider" });
export type AgentOpenrouterProvider = z.infer<typeof AgentOpenrouterProviderSchema>;
