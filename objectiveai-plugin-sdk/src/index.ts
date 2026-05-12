/**
 * objectiveai-plugin-sdk
 *
 * Thin TypeScript shim that lets plugin authors call the host
 * viewer's IPC bridge from inside their iframe-mounted UI bundle,
 * with the same `invoke` / `listen` surface as `@tauri-apps/api`.
 *
 * Production context (loaded inside the host viewer):
 *   - The plugin's `index.html` runs inside an `<iframe sandbox>`
 *     pointed at `tauri://localhost/plugin/<name>/`.
 *   - `window.parent` is the host viewer's React app.
 *   - `invoke()` posts an `{kind: 'invoke', id, method, args}` message
 *     to `window.parent`; the bridge replies with an
 *     `{kind: 'invoke-result', id, result | error}` message.
 *   - `listen()` registers a callback for incoming
 *     `{kind: 'plugin-event', type, value}` messages forwarded by the
 *     bridge from the Rust-side `Event::Plugin` emissions.
 *
 * Dev context (plugin author runs their own Tauri shell standalone):
 *   - `window.parent === window` (no host).
 *   - `invoke()` falls through to `@tauri-apps/api`'s `invoke` if
 *     available, otherwise rejects.
 *   - `listen()` falls through to `@tauri-apps/api`'s `listen`.
 *
 * The plugin author writes the same code in both contexts.
 */

/** Default timeout (ms) for `invoke` round-trips when in iframe mode. */
const INVOKE_TIMEOUT_MS = 30_000;

/** True when running inside an iframe in the host viewer. */
function isInIframe(): boolean {
  return typeof window !== "undefined" && window.parent !== window;
}

interface InvokeMessage {
  kind: "invoke";
  id: string;
  method: string;
  args: unknown;
}

interface InvokeResultMessage {
  kind: "invoke-result";
  id: string;
  result?: unknown;
  error?: string;
}

interface PluginEventMessage {
  kind: "plugin-event";
  type: string;
  value: unknown;
}

type IncomingMessage = InvokeResultMessage | PluginEventMessage;

/** RFC-4122 v4 UUID. Small/local implementation — no extra deps. */
function uuid(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  const bytes = new Uint8Array(16);
  if (typeof crypto !== "undefined" && "getRandomValues" in crypto) {
    crypto.getRandomValues(bytes);
  } else {
    for (let i = 0; i < 16; i++) bytes[i] = Math.floor(Math.random() * 256);
  }
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, "0"));
  return (
    hex.slice(0, 4).join("") +
    "-" +
    hex.slice(4, 6).join("") +
    "-" +
    hex.slice(6, 8).join("") +
    "-" +
    hex.slice(8, 10).join("") +
    "-" +
    hex.slice(10, 16).join("")
  );
}

interface PendingInvoke {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

const pending = new Map<string, PendingInvoke>();
const listeners = new Map<string, Set<(value: unknown) => void>>();
let messageHandlerAttached = false;

function attachMessageHandler(): void {
  if (messageHandlerAttached) return;
  if (typeof window === "undefined") return;
  window.addEventListener("message", (event: MessageEvent) => {
    const msg = event.data as IncomingMessage | null;
    if (!msg || typeof msg !== "object") return;
    if (msg.kind === "invoke-result") {
      const slot = pending.get(msg.id);
      if (!slot) return;
      clearTimeout(slot.timer);
      pending.delete(msg.id);
      if (typeof msg.error === "string") {
        slot.reject(new Error(msg.error));
      } else {
        slot.resolve(msg.result);
      }
    } else if (msg.kind === "plugin-event") {
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
    }
  });
  messageHandlerAttached = true;
}

/**
 * Invoke a host-bridged command. In the iframe context this routes
 * through `window.parent` via postMessage; in standalone-dev context
 * it falls through to `@tauri-apps/api`'s `invoke`.
 */
export async function invoke<T = unknown>(method: string, args: unknown = null): Promise<T> {
  if (!isInIframe()) {
    // Standalone dev mode: try `@tauri-apps/api`.
    try {
      const mod = await import("@tauri-apps/api/core");
      return (await mod.invoke<T>(method, args as Record<string, unknown>)) as T;
    } catch {
      throw new Error(
        `objectiveai-plugin-sdk: invoke('${method}') called outside an iframe and ` +
          `@tauri-apps/api is unavailable.`,
      );
    }
  }
  attachMessageHandler();
  const id = uuid();
  const message: InvokeMessage = { kind: "invoke", id, method, args };
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`objectiveai-plugin-sdk: invoke('${method}') timed out`));
    }, INVOKE_TIMEOUT_MS);
    pending.set(id, {
      resolve: (v) => resolve(v as T),
      reject,
      timer,
    });
    window.parent.postMessage(message, "*");
  });
}

/**
 * Register a handler for incoming plugin events. Returns an
 * unsubscribe function. In iframe context the events come from the
 * host's bridge; in standalone-dev context they come from
 * `@tauri-apps/api`'s `listen`.
 *
 * `type` matches the `type` field of `PluginRequest` emitted by the
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
 * listeners + pending maps are what carry per-test state. */
export function __resetForTests(): void {
  for (const slot of pending.values()) clearTimeout(slot.timer);
  pending.clear();
  listeners.clear();
}
