import { z } from "zod";

export const AgentOpenrouterStopSchema = z.union([z.string().describe("A single stop sequence."), z.array(z.string()).describe("Multiple stop sequences (up to 4 typically supported).")]).describe("Stop sequences that terminate model generation.\n\nWhen the model generates any of these sequences, it immediately\nstops producing further tokens.").meta({ title: "agent.openrouter.Stop" });
export type AgentOpenrouterStop = z.infer<typeof AgentOpenrouterStopSchema>;
