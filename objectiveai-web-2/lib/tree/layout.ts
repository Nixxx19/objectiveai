import type { TreeNode, LayoutNode } from "./types";

/** Node sizing constants */
const NODE_H = 38;
const VC_NODE_H = 48;
const NODE_GAP_X = 12;
const NODE_GAP_Y = 20;

/** Measure a node's width based on its content */
function measureWidth(node: TreeNode): number {
  if (node.kind === "vector-completion") {
    const responses = node.responses ?? [];
    // Cap each pill label at 10 chars for layout; render truncates with CSS
    const MAX_PILL_CHARS = 10;
    const pillsWidth = responses.reduce(
      (sum, r) => sum + Math.min(r.length, MAX_PILL_CHARS) * 6 + 10,
      0
    ) + Math.max(0, responses.length - 1) * 3;
    // 56px = VOTE badge + vcTop padding + gaps
    return Math.max(150, 56 + pillsWidth);
  }
  // Function node: label + score + type
  const typeLen = node.functionType?.length ?? 0;
  return Math.max(120, node.label.length * 6.5 + typeLen * 5 + 52);
}

/** Cached node measurements computed once per layout */
interface NodeMetrics {
  width: number;
  height: number;
  span: number;
}

/**
 * Top-down tree layout. Each subtree occupies a horizontal span
 * equal to the sum of its children spans (or its own width).
 * Nodes are centered above their children.
 */
export function layoutTree(root: TreeNode): LayoutNode {
  // Single bottom-up pass: compute widths, heights, and spans together
  const metrics = new Map<string, NodeMetrics>();
  computeMetrics(root, metrics);

  // Top-down pass: assign positions using cached metrics
  return assignPositions(root, 0, 0, metrics);
}

function computeMetrics(node: TreeNode, metrics: Map<string, NodeMetrics>): NodeMetrics {
  const width = measureWidth(node);
  const height = node.kind === "vector-completion" ? VC_NODE_H : NODE_H;

  if (node.children.length === 0) {
    const m = { width, height, span: width };
    metrics.set(node.id, m);
    return m;
  }

  let childrenSpan = 0;
  for (const child of node.children) {
    childrenSpan += computeMetrics(child, metrics).span;
  }
  childrenSpan += (node.children.length - 1) * NODE_GAP_X;

  const m = { width, height, span: Math.max(width, childrenSpan) };
  metrics.set(node.id, m);
  return m;
}

function assignPositions(
  node: TreeNode,
  left: number,
  top: number,
  metrics: Map<string, NodeMetrics>
): LayoutNode {
  const { width, height, span } = metrics.get(node.id)!;

  // Center this node in its span
  const x = left + (span - width) / 2;
  const y = top;

  const children: LayoutNode[] = [];

  if (node.children.length > 0) {
    let totalChildrenSpan = 0;
    for (const child of node.children) {
      totalChildrenSpan += metrics.get(child.id)!.span;
    }
    totalChildrenSpan += (node.children.length - 1) * NODE_GAP_X;

    let childLeft = left + (span - totalChildrenSpan) / 2;
    const childTop = top + height + NODE_GAP_Y;

    for (const child of node.children) {
      const childSpan = metrics.get(child.id)!.span;
      children.push(assignPositions(child, childLeft, childTop, metrics));
      childLeft += childSpan + NODE_GAP_X;
    }
  }

  return { node, x, y, width, height, children };
}

/** Get the total bounding box of the layout */
export function layoutBounds(root: LayoutNode): {
  width: number;
  height: number;
} {
  let maxX = 0;
  let maxY = 0;

  function walk(ln: LayoutNode) {
    maxX = Math.max(maxX, ln.x + ln.width);
    maxY = Math.max(maxY, ln.y + ln.height);
    for (const child of ln.children) walk(child);
  }

  walk(root);
  return { width: maxX, height: maxY };
}
