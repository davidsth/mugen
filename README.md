# Mugen — Native app browser (early development)

Mugen is an experimental desktop browser replacement. The browser is a Rust native application; apps are separate Crystal web servers that send a typed UI tree. The browser renders the tree and never executes app code.

## Workspace crates

- `crates/shell/` — winit + softbuffer shell, joined-app list, click-to-switch, and local app endpoint registration.
- `crates/renderer/` — renderer groundwork; not yet wired into the shell.
- `crates/layout/` — typed layout groundwork; not yet wired into the shell.
- `crates/protocol/` — protocol v1 types and validation for snapshots, patches, and interaction events.

## Current status

The shell can register a local app endpoint and show a placeholder pane. It does **not** yet connect to the app's WebSocket or render a live Snapshot. The renderer and layout crates remain groundwork. Protocol v1 is a typed contract and validation layer, not a completed interactive runtime.

## Protocol and architecture

Read the repository-root `AGENTS.md` for the architectural contract, `docs/protocol-v1.md` for the wire model, `docs/interaction-model.md` for event flow and local interaction, and `docs/crystal-sdk.md` for the separate app SDK contract. The old `docs/protocol-v0.md` is historical context.

No HTML/CSS/JS, WebView, arbitrary URL navigation, embedded Crystal runtime, or app code inside the browser process.

## Commands

Run from the repository root:

```sh
cargo test -p mugen-protocol
cargo fmt --check
cargo clippy --workspace -- -D warnings
```
