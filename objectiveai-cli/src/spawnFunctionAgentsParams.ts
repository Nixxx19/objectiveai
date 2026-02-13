import { z } from "zod";

export const SpawnFunctionAgentsParamsSchema = z.array(
  z.object({
    name: z.string(),
    spec: z.string(),
    type: z.enum(["scalar.function", "vector.function"]),
  }),
);

export type SpawnFunctionAgentsParams = z.infer<
  typeof SpawnFunctionAgentsParamsSchema
>;
