import { useState, useCallback, useRef, useEffect } from "react";
import type {
  ExecutionTimeline,
  ExecutionFrame,
  NodeExecution,
  NodeState,
} from "./execution";

export interface ExecutionState {
  /** Per-node execution data */
  nodes: Map<string, NodeExecution>;
  /** Current frame index (-1 = not started) */
  frameIndex: number;
  /** Whether auto-playing */
  playing: boolean;
  /** Current frame's label */
  label: string;
  /** Total frames */
  totalFrames: number;
}

const FRAME_INTERVAL = 400; // ms between frames during playback

export function useExecution(timeline: ExecutionTimeline) {
  const [state, setState] = useState<ExecutionState>(() => ({
    nodes: new Map(),
    frameIndex: -1,
    playing: false,
    label: "Ready",
    totalFrames: timeline.frames.length,
  }));

  const playRef = useRef(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const applyFrame = useCallback(
    (frame: ExecutionFrame, nodes: Map<string, NodeExecution>) => {
      const next = new Map(nodes);
      const existing = next.get(frame.nodeId) ?? {
        state: "idle" as NodeState,
        votes: [],
        scores: [],
        weights: [],
      };

      const updated: NodeExecution = {
        ...existing,
        state: frame.state,
      };

      if (frame.vote) {
        updated.votes = [...existing.votes, frame.vote];
      }
      if (frame.scores) {
        updated.scores = frame.scores;
      }
      if (frame.weights) {
        updated.weights = frame.weights;
      }
      if (frame.output !== undefined) {
        updated.output = frame.output;
      }

      next.set(frame.nodeId, updated);
      return next;
    },
    []
  );

  const stepForward = useCallback(() => {
    setState((prev) => {
      const nextIndex = prev.frameIndex + 1;
      if (nextIndex >= timeline.frames.length) {
        playRef.current = false;
        return { ...prev, playing: false };
      }
      const frame = timeline.frames[nextIndex];
      const nodes = applyFrame(frame, prev.nodes);
      return {
        ...prev,
        nodes,
        frameIndex: nextIndex,
        label: frame.label,
      };
    });
  }, [timeline, applyFrame]);

  const play = useCallback(() => {
    playRef.current = true;
    setState((prev) => ({ ...prev, playing: true }));

    function tick() {
      if (!playRef.current) return;
      stepForward();
      timerRef.current = setTimeout(tick, FRAME_INTERVAL);
    }
    tick();
  }, [stepForward]);

  const pause = useCallback(() => {
    playRef.current = false;
    if (timerRef.current) clearTimeout(timerRef.current);
    setState((prev) => ({ ...prev, playing: false }));
  }, []);

  const reset = useCallback(() => {
    playRef.current = false;
    if (timerRef.current) clearTimeout(timerRef.current);
    setState({
      nodes: new Map(),
      frameIndex: -1,
      playing: false,
      label: "Ready",
      totalFrames: timeline.frames.length,
    });
  }, [timeline]);

  // Clear timer on unmount to prevent state updates after cleanup
  useEffect(() => {
    return () => {
      playRef.current = false;
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  return { state, stepForward, play, pause, reset };
}
