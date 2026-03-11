import { z } from "zod";
import { VectorCompletionsRequestProfileEntrySchema } from "./profileEntry";

export const VectorCompletionsRequestProfileSchema = z.union([z.array(z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()])).describe("Simple vector of decimal weights."), z.array(VectorCompletionsRequestProfileEntrySchema).describe("Vector of entries with optional invert flags.")]).describe("Profile weights for a vector completion.\n\nPreviously this was a simple `Vec<Decimal>`. To support per-agent inversion\nwhile remaining backwards compatible, the field is now an untagged enum:\n\n- `Weights(Vec<Decimal>)` - legacy representation (no inversion)\n- `Entries(Vec<ProfileEntry>)` - weights with optional per-agent `invert`").meta({ title: "vector.completions.request.Profile" });
export type VectorCompletionsRequestProfile = z.infer<typeof VectorCompletionsRequestProfileSchema>;
