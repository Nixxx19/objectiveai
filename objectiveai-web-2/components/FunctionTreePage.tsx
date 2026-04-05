"use client";

import { useMemo } from "react";
import { FunctionTree } from "./FunctionTree";
import { ExecutionControls } from "./ExecutionControls";
import {
  nestedScalarSuperBranch,
  buildMockRegistry,
} from "@/lib/tree/examples";
import { buildSimulation } from "@/lib/tree/simulation";
import { useExecution } from "@/lib/tree/useExecution";

export function FunctionTreePage() {
  const registry = useMemo(() => buildMockRegistry(), []);
  const timeline = useMemo(() => buildSimulation(), []);
  const { state, stepForward, play, pause, reset } = useExecution(timeline);

  return (
    <main
      style={{
        minHeight: "100vh",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        padding: "48px 24px 24px",
        gap: 24,
      }}
    >
      <header style={{ textAlign: "center" }}>
        <h1
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: 14,
            color: "var(--info-bright)",
            fontWeight: 500,
            letterSpacing: "0.5px",
          }}
        >
          nested-scalar-super-branch
        </h1>
        <p
          style={{
            fontFamily: "var(--font-sans)",
            fontSize: 12,
            color: "var(--info-dim)",
            marginTop: 4,
          }}
        >
          {nestedScalarSuperBranch.description}
        </p>
      </header>

      <ExecutionControls
        state={state}
        onStep={stepForward}
        onPlay={play}
        onPause={pause}
        onReset={reset}
      />

      <FunctionTree
        name="nested-scalar-super-branch"
        root={nestedScalarSuperBranch}
        registry={registry}
        executions={state.nodes}
      />
    </main>
  );
}
