import { z } from "zod";
import { FunctionsInlineProfileSchema } from "./inlineProfile";
import { FunctionsRemoteProfileSchema } from "./remoteProfile";

export const FunctionsProfileSchema = z.union([FunctionsRemoteProfileSchema.describe("A remote profile with metadata."), FunctionsInlineProfileSchema.describe("An inline profile definition.")]).describe("A Profile definition, either remote or inline.\n\nProfiles contain the weights and nested configurations needed to execute\na Function. They correspond to a Function's task structure.").meta({ title: "functions.Profile" });
export type FunctionsProfile = z.infer<typeof FunctionsProfileSchema>;
