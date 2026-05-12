/**
 * Host-side bridge between plugin iframes and Tauri.
 *
 * Plugin iframes use `@objectiveai/plugin-sdk` to post messages of
 * shape `{kind: 'invoke', id, method, args}` to `window.parent`.
 * This module:
 *   - Validates each message comes from a registered plugin iframe.
 *   - Routes `invoke` calls to the allow-listed Tauri commands.
 *   - Listens for `plugin-<name>` events from the Rust backend and
 *     forwards them as `{kind: 'plugin-event', type, value}`
 *     messages to the matching iframe.
 *
 * Allow-list (commands plugin iframes may call):
 *   - `plugin_invoke` — call the plugin's own backend binary.
 *   - `objectiveai_api_call` — proxied API request (future).
 *
 * Any other command name is rejected with an error reply.
 */
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen, type UnlistenFn } from "@tauri-apps/api/event";

const ALLOWED_COMMANDS: ReadonlySet<string> = new Set([
  "plugin_invoke",
  "objectiveai_api_call",
]);

type IframeHandle = {
  pluginName: string;
  iframe: HTMLIFrameElement;
};

interface InvokeMessage {
  kind: "invoke";
  id: string;
  method: string;
  args: unknown;
}

const iframes = new Map<string, IframeHandle>();
const tauriUnlisteners = new Map<string, UnlistenFn>();
let messageHandlerAttached = false;

/** Register a plugin iframe so the bridge can route to/from it. */
export function registerIframe(pluginName: string, iframe: HTMLIFrameElement): void {
  iframes.set(pluginName, { pluginName, iframe });
  attachMessageHandler();
  void subscribeToPluginEvents(pluginName);
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

function attachMessageHandler(): void {
  if (messageHandlerAttached) return;
  window.addEventListener("message", handleIncomingMessage);
  messageHandlerAttached = true;
}

function handleIncomingMessage(event: MessageEvent): void {
  const data = event.data;
  if (!data || typeof data !== "object") return;
  const msg = data as Partial<InvokeMessage>;
  if (msg.kind !== "invoke") return;
  if (typeof msg.id !== "string" || typeof msg.method !== "string") return;

  // Find which registered iframe this came from.
  const matched = [...iframes.values()].find((h) => h.iframe.contentWindow === event.source);
  if (!matched) {
    // Unknown source — ignore silently. Don't leak that the bridge exists.
    return;
  }

  // Allow-list check.
  if (!ALLOWED_COMMANDS.has(msg.method)) {
    reply(matched, msg.id, undefined, `plugin command not allowed: ${msg.method}`);
    return;
  }

  // For `plugin_invoke`, force the `name` arg to the iframe's plugin
  // name regardless of what the plugin passed — plugins can only
  // invoke themselves, never each other.
  let args = msg.args as Record<string, unknown> | null | undefined;
  if (msg.method === "plugin_invoke") {
    args = { ...(args ?? {}), name: matched.pluginName };
  }

  void (async () => {
    try {
      const result = await tauriInvoke(msg.method as string, args ?? {});
      reply(matched, msg.id as string, result, undefined);
    } catch (e) {
      const err = e instanceof Error ? e.message : String(e);
      reply(matched, msg.id as string, undefined, err);
    }
  })();
}

function reply(
  handle: IframeHandle,
  id: string,
  result: unknown,
  error: string | undefined,
): void {
  const message =
    error === undefined
      ? { kind: "invoke-result", id, result }
      : { kind: "invoke-result", id, error };
  handle.iframe.contentWindow?.postMessage(message, "*");
}

async function subscribeToPluginEvents(pluginName: string): Promise<void> {
  if (tauriUnlisteners.has(pluginName)) return;
  const eventName = `plugin-${pluginName}`;
  const unlisten = await tauriListen<{ plugin: string; request: { type: string; value: unknown } }>(
    eventName,
    (event) => {
      const handle = iframes.get(pluginName);
      if (!handle) return;
      const payload = event.payload?.request;
      if (!payload) return;
      handle.iframe.contentWindow?.postMessage(
        { kind: "plugin-event", type: payload.type, value: payload.value },
        "*",
      );
    },
  );
  tauriUnlisteners.set(pluginName, unlisten);
}
