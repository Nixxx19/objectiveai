"use client";

import type { ExecutionState } from "@/lib/tree/useExecution";
import styles from "./FunctionTree.module.css";

interface Props {
  state: ExecutionState;
  onStep: () => void;
  onPlay: () => void;
  onPause: () => void;
  onReset: () => void;
}

export function ExecutionControls({
  state,
  onStep,
  onPlay,
  onPause,
  onReset,
}: Props) {
  const done = state.frameIndex >= state.totalFrames - 1;

  return (
    <div className={styles.controls}>
      {state.playing ? (
        <button className={styles.controlBtn} onClick={onPause}>
          pause
        </button>
      ) : (
        <button
          className={styles.controlBtn}
          onClick={onPlay}
          disabled={done}
        >
          play
        </button>
      )}
      <button
        className={styles.controlBtn}
        onClick={onStep}
        disabled={state.playing || done}
      >
        step
      </button>
      <button className={styles.controlBtn} onClick={onReset}>
        reset
      </button>
      <span className={styles.frameLabel}>{state.label}</span>
      <span className={styles.frameCounter}>
        {Math.max(0, state.frameIndex + 1)}/{state.totalFrames}
      </span>
    </div>
  );
}
