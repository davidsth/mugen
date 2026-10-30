# browser/ — The Mugen Browser (Rust)

This is the desktop app users install. It renders apps. It never runs
app code, never loads HTML/CSS/JS, and never opens arbitrary URLs.

This folder is the whole repo's Rust workspace. There is no Crystal
code in this repo: apps live in separate repos and run as their own
web servers.

## Crates

- `crates/shell/` — the runnable browser: winit + softbuffer window
  (v0.1), joined-app list, click-to-switch, local-app parsing
  (`--app <host:port>` / `MUGEN_LOCAL_APP`), lifecycle placeholders.
- `crates/renderer/` — wgpu display list, batching, texture atlas,
  culling (Step 3). Text via cosmic-text/HarfBuzz underneath;
  accessibility via accesskit. Do not write a GPU backend or a text
  shaper here. It does not yet replace softbuffer in the shell.
- `crates/layout/` — typed constraint layout (Row/Column/Stack/Grid),
  two-pass measure/place, dirty tracking (Step 3). Typed values only,
  no CSS.
- `crates/protocol/` — Snapshot, Patch, Event, and hard validation.
  The Crystal side mirrors this protocol in its own repo; it does
  not import this crate.

## Commands (run from this folder, not the repo root)

```sh
cargo test -p mugen-protocol
cargo run -p mugen-shell
cargo run -p mugen-shell -- --app 127.0.0.1:4000
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

## Adding a local app

Run the app's server first (see its own repo — e.g. the demo app on
`127.0.0.1:4000`), then pass its IP:port to the browser as above.
The browser logs `manifest: http://<endpoint>/manifest` and
`websocket: ws://<endpoint>/ws` and adds a sidebar icon.

v0.1 limitation, stated plainly: the shell registers the endpoint
and shows a placeholder pane. It does not yet fetch/render the
app's Snapshot over WebSocket. That is Step 5.

## Rules for this folder

- Shell owns only: discovery, join/leave, list, switching, lifecycle,
  auth, permissions, storage. Everything inside an app window belongs
  to the app.
- Validate every tree in `protocol` before layout. Reject bad trees,
  keep the shell alive.
- No WebView, no address bar, no ad/popup node types.
- See the repo-root `AGENTS.md` for the full boundary and protocol rules.
