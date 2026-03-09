"use client";

import { useState, useEffect, useRef } from "react";
import { Functions } from "objectiveai";
import { createPublicClient } from "@/lib/client";
import type { InputFunctionDefinition } from "@objectiveai/function-tree";

const MAX_DEPTH = 3;

/**
 * Recursively fetches sub-function definitions for nested function tasks.
 * Returns a Map<"owner/repo", InputFunctionDefinition> for the tree's resolvedSubFunctions prop.
 */
export function useResolvedSubFunctions(
  definition: InputFunctionDefinition | null | undefined,
): Map<string, InputFunctionDefinition> | undefined {
  const [resolved, setResolved] = useState<Map<string, InputFunctionDefinition> | undefined>(undefined);
  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    if (!definition || !definition.tasks || definition.tasks.length === 0) {
      setResolved(undefined);
      return;
    }

    // Check if there are any nested function tasks
    const hasFunctionTasks = definition.tasks.some(
      (t) => t.type === "scalar.function" || t.type === "vector.function",
    );
    if (!hasFunctionTasks) {
      setResolved(undefined);
      return;
    }

    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    const client = createPublicClient();
    const result = new Map<string, InputFunctionDefinition>();
    const visited = new Set<string>();

    async function resolve(
      def: InputFunctionDefinition,
      depth: number,
    ): Promise<void> {
      if (depth >= MAX_DEPTH || controller.signal.aborted) return;

      const fetchPromises: Promise<void>[] = [];

      for (const task of def.tasks) {
        if (
          (task.type === "scalar.function" || task.type === "vector.function") &&
          task.owner &&
          task.repository
        ) {
          const key = `${task.owner}/${task.repository}`;
          if (visited.has(key)) continue;
          visited.add(key);

          fetchPromises.push(
            (async () => {
              try {
                const subDef = await Functions.retrieve(
                  client,
                  "github",
                  task.owner!,
                  task.repository!,
                  task.commit ?? null,
                  { signal: controller.signal },
                );
                if (controller.signal.aborted) return;

                const asDef = subDef as unknown as InputFunctionDefinition;
                result.set(key, asDef);

                // Recurse
                if (asDef.tasks && depth + 1 < MAX_DEPTH) {
                  await resolve(asDef, depth + 1);
                }
              } catch {
                // Sub-function not found or inaccessible — leave as unexpanded node
              }
            })(),
          );
        }
      }

      await Promise.all(fetchPromises);
    }

    resolve(definition, 0).then(() => {
      if (!controller.signal.aborted && result.size > 0) {
        setResolved(new Map(result));
      }
    });

    return () => controller.abort();
  }, [definition]);

  return resolved;
}
