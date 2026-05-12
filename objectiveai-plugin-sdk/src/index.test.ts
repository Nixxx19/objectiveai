import { describe, expect, it, beforeEach, afterEach, vi } from "vitest";
import { invoke, listen, __resetForTests } from "./index";

/**
 * Simulate the iframe context by mocking `window.parent` to be a
 * distinct object that proxies postMessage to our test harness. The
 * SDK's `isInIframe()` check is just `window.parent !== window`.
 */
function setupIframeContext() {
  __resetForTests();
  const parentMessages: unknown[] = [];
  const parent = {
    postMessage: (msg: unknown) => parentMessages.push(msg),
  };
  // Override window.parent for the duration of the test.
  Object.defineProperty(window, "parent", { value: parent, configurable: true });
  return {
    parentMessages,
    /** Simulate a message arriving from the parent. */
    deliver(msg: unknown) {
      window.dispatchEvent(new MessageEvent("message", { data: msg }));
    },
  };
}

function teardownIframeContext() {
  Object.defineProperty(window, "parent", { value: window, configurable: true });
  __resetForTests();
}

describe("invoke in iframe context", () => {
  let ctx: ReturnType<typeof setupIframeContext>;
  beforeEach(() => {
    ctx = setupIframeContext();
  });
  afterEach(teardownIframeContext);

  it("posts an invoke message and resolves on matching result", async () => {
    const promise = invoke<{ ok: boolean }>("test_method", { a: 1 });
    expect(ctx.parentMessages).toHaveLength(1);
    const msg = ctx.parentMessages[0] as {
      kind: string;
      id: string;
      method: string;
      args: unknown;
    };
    expect(msg.kind).toBe("invoke");
    expect(msg.method).toBe("test_method");
    expect(msg.args).toEqual({ a: 1 });

    ctx.deliver({ kind: "invoke-result", id: msg.id, result: { ok: true } });
    await expect(promise).resolves.toEqual({ ok: true });
  });

  it("rejects on error result", async () => {
    const promise = invoke("test_method");
    const msg = ctx.parentMessages[0] as { id: string };
    ctx.deliver({ kind: "invoke-result", id: msg.id, error: "kaboom" });
    await expect(promise).rejects.toThrow("kaboom");
  });

  it("times out when no response arrives", async () => {
    vi.useFakeTimers();
    const promise = invoke("test_method");
    vi.advanceTimersByTime(31_000);
    await expect(promise).rejects.toThrow(/timed out/);
    vi.useRealTimers();
  });
});

describe("listen in iframe context", () => {
  let ctx: ReturnType<typeof setupIframeContext>;
  beforeEach(() => {
    ctx = setupIframeContext();
  });
  afterEach(teardownIframeContext);

  it("fires the handler when a matching plugin-event arrives", () => {
    const calls: unknown[] = [];
    listen<{ x: number }>("my_event", (v) => calls.push(v));
    ctx.deliver({ kind: "plugin-event", type: "my_event", value: { x: 42 } });
    expect(calls).toEqual([{ x: 42 } as unknown]);
  });

  it("ignores plugin-events with other types", () => {
    const calls: unknown[] = [];
    listen("my_event", (v) => calls.push(v));
    ctx.deliver({ kind: "plugin-event", type: "other_event", value: 1 });
    expect(calls).toEqual([]);
  });

  it("returns an unlisten that stops further events", () => {
    const calls: unknown[] = [];
    const unlisten = listen("my_event", (v) => calls.push(v));
    ctx.deliver({ kind: "plugin-event", type: "my_event", value: 1 });
    unlisten();
    ctx.deliver({ kind: "plugin-event", type: "my_event", value: 2 });
    expect(calls).toEqual([1]);
  });
});
