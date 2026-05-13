/**
 * objectiveai-plugin-sdk
 *
 * Thin TypeScript shim that lets plugin authors subscribe to events
 * emitted by the host viewer from inside their iframe-mounted UI
 * bundle, with the same `listen` surface as `@tauri-apps/api`.
 *
 * Production context (loaded inside the host viewer):
 *   - The plugin's `index.html` runs inside an `<iframe sandbox>`
 *     pointed at `plugin://localhost/<repo>/`.
 *   - `window.parent` is the host viewer's React app.
 *   - `listen()` registers a callback for incoming
 *     `{kind: 'plugin-event', type, value}` messages forwarded by the
 *     bridge from the Rust-side `Event::Plugin` emissions.
 *
 * Dev context (plugin author runs their own Tauri shell standalone):
 *   - `window.parent === window` (no host).
 *   - `listen()` falls through to `@tauri-apps/api`'s `listen`.
 *
 * The plugin author writes the same code in both contexts.
 */

/** True when running inside an iframe in the host viewer. */
function isInIframe(): boolean {
  return typeof window !== "undefined" && window.parent !== window;
}

interface PluginEventMessage {
  kind: "plugin-event";
  type: string;
  value: unknown;
}

const listeners = new Map<string, Set<(value: unknown) => void>>();
let messageHandlerAttached = false;

function attachMessageHandler(): void {
  if (messageHandlerAttached) return;
  if (typeof window === "undefined") return;
  window.addEventListener("message", (event: MessageEvent) => {
    const msg = event.data as PluginEventMessage | null;
    if (!msg || typeof msg !== "object") return;
    if (msg.kind !== "plugin-event") return;
    const set = listeners.get(msg.type);
    if (!set) return;
    for (const fn of set) {
      try {
        fn(msg.value);
      } catch (e) {
        // Don't let one handler take down the others.
        // eslint-disable-next-line no-console
        console.error("objectiveai-plugin-sdk listener threw:", e);
      }
    }
  });
  messageHandlerAttached = true;
}

/**
 * Register a handler for incoming plugin events. Returns an
 * unsubscribe function. In iframe context the events come from the
 * host's bridge; in standalone-dev context they come from
 * `@tauri-apps/api`'s `listen`.
 *
 * `type` matches the `type` field of the `Event` emitted by the
 * Rust backend — the string the plugin author registered in their
 * manifest's `viewer_routes` entry.
 */
export function listen<T = unknown>(
  type: string,
  handler: (value: T) => void,
): () => void {
  if (!isInIframe()) {
    // Standalone dev mode: register via @tauri-apps/api.
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    void (async () => {
      try {
        const mod = await import("@tauri-apps/api/event");
        if (cancelled) return;
        const u = await mod.listen<{ value: T }>(`plugin-${type}`, (e) =>
          handler(e.payload?.value as T),
        );
        if (cancelled) {
          u();
        } else {
          unlisten = u;
        }
      } catch {
        // eslint-disable-next-line no-console
        console.warn(
          `objectiveai-plugin-sdk: listen('${type}') called outside an iframe and ` +
            `@tauri-apps/api is unavailable; events will not fire.`,
        );
      }
    })();
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }
  attachMessageHandler();
  const set = listeners.get(type) ?? new Set();
  const fn = (value: unknown) => handler(value as T);
  set.add(fn);
  listeners.set(type, set);
  return () => {
    const s = listeners.get(type);
    if (!s) return;
    s.delete(fn);
    if (s.size === 0) listeners.delete(type);
  };
}

/** Internal-use: clear in-flight state. Exposed for tests only.
 * Note: the module-level `message` event listener stays attached —
 * removing/re-attaching it would just register a duplicate. The
 * listeners map is what carries per-test state. */
export function __resetForTests(): void {
  listeners.clear();
}
