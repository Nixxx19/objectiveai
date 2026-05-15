/**
 * Host-side bridge between plugin iframes and Tauri.
 *
 * For each registered plugin iframe, this module subscribes to the
 * matching `<repository>` Tauri channel and forwards each emitted
 * `Event` into the iframe as a `{kind: 'plugin-event', ...}`
 * postMessage. The iframe consumes those via `@objectiveai/sdk/viewer`'s
 * `listen()` (or by adding its own `window.addEventListener('message')`).
 *
 * Two event variants flow host -> iframe:
 *
 *   - `inbound` — host has data for the plugin (the existing path).
 *     The iframe's `listen(sub_type, handler)` matches on the
 *     `sub_type` discriminator.
 *
 *   - `cli_command` — one line of stdout from an in-process
 *     `objectiveai_cli::run()` invocation that this iframe started
 *     via `invokeCli`. No sub_type.
 *
 * The reverse direction (iframe -> host) carries `cli-invoke`
 * postMessages; this module catches them, resolves the originating
 * iframe via `MessageEvent.source`, and dispatches the Tauri command
 * `cli_run` with the originator's plugin name as `origin`. Messages
 * from unknown sources are dropped (security: don't let a random
 * iframe drive the host CLI without identity).
 */
import { invoke } from "@tauri-apps/api/core";
import { listen as tauriListen, type UnlistenFn } from "@tauri-apps/api/event";

type IframeHandle = {
  pluginName: string;
  iframe: HTMLIFrameElement;
};

type InboundPayload = {
  type: "inbound";
  destination: string;
  sub_type: string;
  value: unknown;
};

type CliCommandPayload = {
  type: "cli_command";
  destination: string;
  value: unknown;
};

type EventPayload = InboundPayload | CliCommandPayload;

const iframes = new Map<string, IframeHandle>();
const tauriUnlisteners = new Map<string, UnlistenFn>();

/** Register a plugin iframe so the bridge can forward events to it. */
export function registerIframe(pluginName: string, iframe: HTMLIFrameElement): void {
  iframes.set(pluginName, { pluginName, iframe });
  void subscribeToPluginEvents(pluginName);
  ensureReverseListener();
}

/** Unregister a previously-registered iframe. Cancels its event sub. */
export function unregisterIframe(pluginName: string): void {
  iframes.delete(pluginName);
  const unlisten = tauriUnlisteners.get(pluginName);
  if (unlisten) {
    unlisten();
    tauriUnlisteners.delete(pluginName);
  }
}

async function subscribeToPluginEvents(pluginName: string): Promise<void> {
  if (tauriUnlisteners.has(pluginName)) return;
  const unlisten = await tauriListen<EventPayload>(pluginName, (event) => {
    const handle = iframes.get(pluginName);
    if (!handle) return;
    const payload = event.payload;
    if (!payload || !payload.type) return;
    if (payload.type === "inbound") {
      handle.iframe.contentWindow?.postMessage(
        {
          kind: "plugin-event",
          type: "inbound",
          sub_type: payload.sub_type,
          value: payload.value,
        },
        "*",
      );
    } else if (payload.type === "cli_command") {
      handle.iframe.contentWindow?.postMessage(
        { kind: "plugin-event", type: "cli_command", value: payload.value },
        "*",
      );
    }
  });
  tauriUnlisteners.set(pluginName, unlisten);
}

// ── Reverse channel: iframe -> host postMessages ──────────────────

let reverseListenerAttached = false;

function ensureReverseListener(): void {
  if (reverseListenerAttached) return;
  if (typeof window === "undefined") return;
  window.addEventListener("message", onIframeMessage);
  reverseListenerAttached = true;
}

function onIframeMessage(event: MessageEvent): void {
  const msg = event.data as { kind?: string; args?: unknown } | null;
  if (!msg || typeof msg !== "object") return;
  if (msg.kind !== "cli-invoke") return;
  if (!Array.isArray(msg.args)) return;

  // Identify which iframe sent this. Drop unidentified sources.
  const origin = findPluginByWindow(event.source);
  if (!origin) return;

  const args = msg.args.filter((a): a is string => typeof a === "string");
  // Fire-and-forget. The host streams cli_command events back via the
  // events bus; the iframe consumes them through its async iterator.
  void invoke("cli_run", { args, origin });
}

function findPluginByWindow(source: MessageEventSource | null): string | null {
  if (!source) return null;
  for (const [name, handle] of iframes) {
    if (handle.iframe.contentWindow === source) return name;
  }
  return null;
}
