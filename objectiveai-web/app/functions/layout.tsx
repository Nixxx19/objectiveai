import type { Metadata } from "next";
import { ReactNode } from "react";

export const metadata: Metadata = {
  title: "Functions",
  description:
    "Browse and execute ObjectiveAI scoring functions. Swarms of AI agents vote with weighted probabilities to produce confidence-scored outputs.",
};

export default function FunctionsLayout({ children }: { children: ReactNode }) {
  return children;
}
