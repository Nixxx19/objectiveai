"use client";

import { useMemo, useRef, useEffect, useCallback, useState, type ReactNode } from "react";
import type { LayoutNode, FunctionDef, TreeNode } from "@/lib/tree/types";
import type { NodeExecution } from "@/lib/tree/execution";
import { buildTree } from "@/lib/tree/build";
import { layoutTree, layoutBounds } from "@/lib/tree/layout";
import styles from "./FunctionTree.module.css";

interface Props {
  name: string;
  root: FunctionDef;
  registry: Map<string, FunctionDef>;
  executions?: Map<string, NodeExecution>;
  onNodeClick?: (node: TreeNode) => void;
  selectedNodeId?: string;
}

export function FunctionTree({ name, root, registry, executions, onNodeClick, selectedNodeId }: Props) {
  const { layout, bounds } = useMemo(() => {
    const tree = buildTree(name, root, registry);
    const l = layoutTree(tree);
    return { layout: l, bounds: layoutBounds(l) };
  }, [name, root, registry]);

  const pad = 32;
  const containerRef = useRef<HTMLDivElement>(null);

  const scrollToCenter = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    const canvasWidth = bounds.width + pad * 2;
    const overflow = canvasWidth - el.clientWidth;
    if (overflow > 0) {
      el.scrollLeft = overflow / 2;
    }
  }, [bounds.width]);

  useEffect(() => {
    scrollToCenter();
  }, [scrollToCenter]);

  return (
    <div ref={containerRef} className={styles.container}>
      <div
        className={styles.canvas}
        style={{
          width: bounds.width + pad * 2,
          height: bounds.height + pad * 2,
          position: "relative",
        }}
      >
        <svg
          className={styles.connectors}
          width={bounds.width + pad * 2}
          height={bounds.height + pad * 2}
          style={{ position: "absolute", top: 0, left: 0 }}
          aria-hidden="true"
        >
          <Connectors node={layout} pad={pad} executions={executions} />
        </svg>
        <Nodes node={layout} pad={pad} executions={executions} onNodeClick={onNodeClick} selectedNodeId={selectedNodeId} />
      </div>
    </div>
  );
}

function Connectors({
  node,
  pad,
  executions,
}: {
  node: LayoutNode;
  pad: number;
  executions?: Map<string, NodeExecution>;
}) {
  const paths: ReactNode[] = [];

  function walk(ln: LayoutNode) {
    const parentCx = ln.x + ln.width / 2 + pad;
    const parentBottom = ln.y + ln.height + pad;

    for (const child of ln.children) {
      const childCx = child.x + child.width / 2 + pad;
      const childTop = child.y + pad;
      const midY = parentBottom + (childTop - parentBottom) / 2;

      const childState = executions?.get(child.node.id)?.state;
      const connectorClass =
        childState === "voting" || childState === "pending"
          ? styles.connectorActive
          : childState === "resolved"
            ? styles.connectorResolved
            : "";

      paths.push(
        <path
          key={`${ln.node.id}-${child.node.id}`}
          d={`M ${parentCx} ${parentBottom} L ${parentCx} ${midY} L ${childCx} ${midY} L ${childCx} ${childTop}`}
          className={`${styles.connector} ${connectorClass}`}
        />
      );

      walk(child);
    }
  }

  walk(node);
  return <>{paths}</>;
}

function Nodes({
  node,
  pad,
  executions,
  onNodeClick,
  selectedNodeId,
}: {
  node: LayoutNode;
  pad: number;
  executions?: Map<string, NodeExecution>;
  onNodeClick?: (node: TreeNode) => void;
  selectedNodeId?: string;
}) {
  const elements: ReactNode[] = [];

  function walk(ln: LayoutNode) {
    const exec = executions?.get(ln.node.id);
    const stateClass = exec ? styles[exec.state] : "";
    const isSelected = selectedNodeId === ln.node.id;
    const clickable = onNodeClick && ln.node.taskMeta;

    elements.push(
      <div
        key={ln.node.id}
        className={`${styles.node} ${styles[ln.node.kind]} ${stateClass} ${isSelected ? styles.nodeSelected : ""} ${clickable ? styles.nodeClickable : ""}`}
        style={{
          left: ln.x + pad,
          top: ln.y + pad,
          width: ln.width,
          height: ln.height,
        }}
        onClick={clickable ? () => onNodeClick(ln.node) : undefined}
      >
        {ln.node.kind === "function" ? (
          <FunctionNode
            label={ln.node.label}
            functionType={ln.node.functionType}
            mapped={ln.node.mapped}
            exec={exec}
          />
        ) : (
          <VectorCompletionNode
            responses={ln.node.responses}
            mapped={ln.node.mapped}
            exec={exec}
          />
        )}
      </div>
    );

    for (const child of ln.children) walk(child);
  }

  walk(node);
  return <>{elements}</>;
}

function FunctionNode({
  label,
  functionType,
  mapped,
  exec,
}: {
  label: string;
  functionType?: string;
  mapped?: boolean;
  exec?: NodeExecution;
}) {
  return (
    <>
      <span className={styles.nodeLabel}>
        {mapped && <span className={styles.mapped}>map</span>}
        {label}
      </span>
      <span className={styles.nodeType}>
        {exec?.output !== undefined && (
          <span className={styles.score}>{exec.output.toFixed(3)}</span>
        )}
        <span className={styles.typeLabel}>{functionType}</span>
      </span>
    </>
  );
}

function VectorCompletionNode({
  responses,
  mapped,
  exec,
}: {
  responses?: string[];
  mapped?: boolean;
  exec?: NodeExecution;
}) {
  const hasScores = exec && exec.scores.length > 0;
  const hasVotes = exec?.votes && exec.votes.length > 0;
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      className={`${styles.vcInner} ${hasVotes ? styles.vcClickable : ""}`}
      onClick={() => hasVotes && setExpanded(!expanded)}
    >
      <div className={styles.vcTop}>
        <span className={styles.nodeLabel}>
          {mapped && <span className={styles.mapped}>map</span>}
          {exec?.votes && exec.votes.length > 0 ? (
            <span className={styles.voteCount}>{exec.votes.length}</span>
          ) : null}
          vote
        </span>
        {responses && responses.length > 0 && !hasScores && (
          <span className={styles.responses}>
            {responses.map((r, i) => (
              <span key={i} className={styles.response} title={r}>{r}</span>
            ))}
          </span>
        )}
        {hasScores && (
          <span className={styles.responses}>
            {responses?.map((r, i) => {
              const pct = (exec.scores[i] * 100).toFixed(0);
              return (
                <span
                  key={i}
                  className={styles.responseWithScore}
                  style={{ opacity: 0.4 + exec.scores[i] * 0.6 }}
                >
                  <span className={styles.responseLabel} title={r}>{r}</span>
                  <span className={styles.responseScore}>{pct}</span>
                </span>
              );
            })}
          </span>
        )}
      </div>
      {hasScores && (
        <div className={styles.distBar}>
          {exec.scores.map((s, i) => (
            <div
              key={i}
              className={styles.distSegment}
              style={{ flex: Math.max(s, 0.01), opacity: 0.3 + s * 0.7 }}
            />
          ))}
        </div>
      )}
      {expanded && exec?.votes && (
        <div className={styles.agentPanel} onClick={(e) => e.stopPropagation()}>
          {exec.votes.map((v, i) => (
            <div key={i} className={styles.agentRow}>
              <span className={styles.agentModel}>{v.model.split("/").pop()}</span>
              <div className={styles.agentBar}>
                {v.vote.map((p, j) => (
                  <div
                    key={j}
                    className={styles.agentSegment}
                    style={{ flex: Math.max(p, 0.005), opacity: 0.2 + p * 0.8 }}
                  />
                ))}
              </div>
              <span className={styles.agentWeight}>{v.weight.toFixed(1)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
