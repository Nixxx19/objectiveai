/**
 * Host-side bridge between plugin iframes and Tauri.
 *
 * For each registered plugin iframe, this module subscribes to the
 * matching `<repository>` Tauri channel and forwards each emitted
 * event into the iframe as a `{kind: 'plugin-event', type, value}`
 * postMessage. The iframe consumes those via `objectiveai-sdk/viewer`'s
 * `listen()` (or by adding its own `window.addEventListener('message')`).
 */
import { listen as tauriListen, type UnlistenFn } from "@tauri-apps/api/event";

type IframeHandle = {
  pluginName: string;
  iframe: HTMLIFrameElement;
};

const iframes = new Map<string, IframeHandle>();
const tauriUnlisteners = new Map<string, UnlistenFn>();

/** Register a plugin iframe so the bridge can forward events to it. */
export function registerIframe(pluginName: string, iframe: HTMLIFrameElement): void {
  iframes.set(pluginName, { pluginName, iframe });
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

async function subscribeToPluginEvents(pluginName: string): Promise<void> {
  if (tauriUnlisteners.has(pluginName)) return;
  const unlisten = await tauriListen<{ destination: string; type: string; value: unknown }>(
    pluginName,
    (event) => {
      const handle = iframes.get(pluginName);
      if (!handle) return;
      const payload = event.payload;
      if (!payload || !payload.type) return;
      handle.iframe.contentWindow?.postMessage(
        { kind: "plugin-event", type: payload.type, value: payload.value },
        "*",
      );
    },
  );
  tauriUnlisteners.set(pluginName, unlisten);
}
