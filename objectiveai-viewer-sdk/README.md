# @objectiveai/viewer-sdk

Plugin SDK for the [ObjectiveAI](https://objectiveai.dev) viewer. A thin TypeScript shim that lets plugin authors subscribe to events emitted by the host viewer from inside their iframe-mounted UI bundle, with the same `listen` surface as `@tauri-apps/api`.

Production context (loaded inside the host viewer): the plugin's `index.html` runs inside an `<iframe sandbox>` pointed at `plugin://localhost/<repo>/`. `window.parent` is the host viewer's React app. `listen()` registers a callback for incoming `{kind: 'plugin-event', type, value}` messages forwarded by the bridge.

Dev context (plugin author runs their own Tauri shell standalone): `window.parent === window` (no host). `listen()` falls through to `@tauri-apps/api`'s `listen`.

The plugin author writes the same code in both contexts.

## Links

- Homepage: <https://objectiveai.dev>
- Repository: <https://github.com/ObjectiveAI/objectiveai>
