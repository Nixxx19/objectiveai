import { AgentEvent } from "./events";

interface AgentPanel {
  name: string;
  lines: string[];
}

export class Dashboard {
  private panels: Map<string, AgentPanel> = new Map();
  private knownNames: Set<string> = new Set();
  private lastRenderedHeight = 0;
  private maxLines: number;
  private dirty = false;
  private renderTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(maxLines = 5) {
    this.maxLines = maxLines;
    // Create root panel
    this.panels.set("", { name: "unnamed function", lines: [] });
  }

  setRootName(name: string): void {
    const panel = this.panels.get("");
    if (panel) panel.name = name;
    this.scheduleRender();
  }

  handleEvent(evt: AgentEvent): void {
    switch (evt.event) {
      case "start": {
        if (!this.panels.has(evt.path)) {
          this.panels.set(evt.path, { name: evt.path, lines: [] });
        }
        this.knownNames.add(evt.path.split("/").pop()!);
        break;
      }
      case "name": {
        const panel = this.panels.get(evt.path);
        if (panel) {
          // Update panel name: replace last segment with actual name
          const parts = evt.path.split("/");
          parts[parts.length - 1] = evt.name;
          panel.name = parts.join("/");
          // Re-key if this is root
          if (evt.path === "") panel.name = evt.name;
        }
        break;
      }
      case "log": {
        let panel = this.panels.get(evt.path);
        if (!panel) {
          panel = { name: evt.path, lines: [] };
          this.panels.set(evt.path, panel);
        }
        // Split multi-line log entries
        const logLines = evt.line.split("\n");
        for (const l of logLines) {
          panel.lines.push(l);
          if (panel.lines.length > this.maxLines) {
            panel.lines.shift();
          }
        }
        break;
      }
      case "done": {
        this.panels.delete(evt.path);
        this.renderNow();
        return;
      }
    }
    this.scheduleRender();
  }

  private scheduleRender(): void {
    this.dirty = true;
    if (this.renderTimer) return;
    this.renderTimer = setTimeout(() => {
      this.renderTimer = null;
      if (this.dirty) this.renderNow();
    }, 50);
  }

  private renderNow(): void {
    this.dirty = false;

    // Build output
    const sections: string[] = [];

    // Root panel first
    const root = this.panels.get("");
    if (root) {
      sections.push(this.formatPanel(root));
    }

    // Children sorted by path
    const sortedPaths = [...this.panels.keys()]
      .filter((p) => p !== "")
      .sort();
    for (const path of sortedPaths) {
      const panel = this.panels.get(path)!;
      sections.push(this.formatPanel(panel));
    }

    const output = sections.join("\n\n") + "\n";
    const newHeight = output.split("\n").length;

    // Clear previous render
    if (this.lastRenderedHeight > 0) {
      process.stdout.write(`\x1b[${this.lastRenderedHeight}A\x1b[0J`);
    }

    process.stdout.write(output);
    this.lastRenderedHeight = newHeight;
  }

  private formatPanel(panel: AgentPanel): string {
    const header = `\x1b[1m=== ${panel.name} ===\x1b[0m`;
    const lines = panel.lines.map((l) => `  ${l}`);
    return [header, ...lines].join("\n");
  }

  findPathByName(name: string): string | undefined {
    for (const [path] of this.panels) {
      if (!path) continue;
      const lastSegment = path.split("/").pop()!;
      if (lastSegment === name) return path;
    }
    return undefined;
  }

  isKnownName(name: string): boolean {
    return this.knownNames.has(name);
  }

  dispose(): void {
    if (this.renderTimer) {
      clearTimeout(this.renderTimer);
      this.renderTimer = null;
    }
  }
}
