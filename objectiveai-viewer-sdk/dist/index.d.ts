/**
 * objectiveai-viewer-sdk
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
export declare function listen<T = unknown>(type: string, handler: (value: T) => void): () => void;
/** Internal-use: clear in-flight state. Exposed for tests only.
 * Note: the module-level `message` event listener stays attached —
 * removing/re-attaching it would just register a duplicate. The
 * listeners map is what carries per-test state. */
export declare function __resetForTests(): void;
//# sourceMappingURL=index.d.ts.map