// src/viewer/index.ts
function isInIframe() {
  return typeof window !== "undefined" && window.parent !== window;
}
var listeners = /* @__PURE__ */ new Map();
var messageHandlerAttached = false;
function attachMessageHandler() {
  if (messageHandlerAttached) return;
  if (typeof window === "undefined") return;
  window.addEventListener("message", (event) => {
    const msg = event.data;
    if (!msg || typeof msg !== "object") return;
    if (msg.kind !== "plugin-event") return;
    const set = listeners.get(msg.type);
    if (!set) return;
    for (const fn of set) {
      try {
        fn(msg.value);
      } catch (e) {
        console.error("@objectiveai/sdk/viewer listener threw:", e);
      }
    }
  });
  messageHandlerAttached = true;
}
function listen(type, handler) {
  if (!isInIframe()) {
    let unlisten = null;
    let cancelled = false;
    void (async () => {
      try {
        const mod = await import('./event-MA6ITXCE.js');
        if (cancelled) return;
        const u = await mod.listen(
          `plugin-${type}`,
          (e) => handler(e.payload?.value)
        );
        if (cancelled) {
          u();
        } else {
          unlisten = u;
        }
      } catch {
        console.warn(
          `@objectiveai/sdk/viewer: listen('${type}') called outside an iframe and @tauri-apps/api is unavailable; events will not fire.`
        );
      }
    })();
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }
  attachMessageHandler();
  const set = listeners.get(type) ?? /* @__PURE__ */ new Set();
  const fn = (value) => handler(value);
  set.add(fn);
  listeners.set(type, set);
  return () => {
    const s = listeners.get(type);
    if (!s) return;
    s.delete(fn);
    if (s.size === 0) listeners.delete(type);
  };
}
function __resetForTests() {
  listeners.clear();
}

export { __resetForTests, listen };
