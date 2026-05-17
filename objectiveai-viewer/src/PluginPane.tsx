import { useEffect, useRef, type ReactElement } from "react";
import type { ViewerPluginInfo } from "./App";
import { registerIframe, unregisterIframe } from "./plugin-bridge";

interface PluginPaneProps {
  info: ViewerPluginInfo;
}

/**
 * Renders a sandboxed iframe pointing at the plugin's UI. The Rust
 * side already resolved the iframe `src=` — either
 * `plugin://localhost/<name>/index.html` for a zip-installed bundle
 * served by the custom URI scheme handler, or the manifest's
 * `viewer_url` verbatim for a remote-URL plugin. Registers the iframe
 * with the postMessage bridge on mount (so the bridge can derive the
 * iframe's target origin from the src), unregisters on unmount.
 *
 * Inactive plugin panes are unmounted (not just hidden) so their
 * iframe memory is reclaimed; performance follow-up if hot-switching
 * between many plugins becomes a UX issue.
 */
export function PluginPane({ info }: PluginPaneProps): ReactElement {
  const ref = useRef<HTMLIFrameElement | null>(null);

  useEffect(() => {
    const iframe = ref.current;
    if (!iframe) return;
    registerIframe(info.name, iframe, info.iframe_src);
    return () => unregisterIframe(info.name);
  }, [info.name, info.iframe_src]);

  return (
    <iframe
      ref={ref}
      title={info.name}
      src={info.iframe_src}
      sandbox="allow-scripts allow-forms"
      style={{
        flex: 1,
        width: "100%",
        height: "100%",
        border: "none",
      }}
    />
  );
}
