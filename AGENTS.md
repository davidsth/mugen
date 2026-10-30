# AGENTS.md — How to work on Mugen (browser repo)

Read this before changing anything. It is the contract for any agent
(Muse, Claude Code, or a human contributor) working in this repo.

## 1. What Mugen is

Mugen (Japanese: infinite / limitless) is a desktop browser replacement.

- The **Mugen browser** is a Rust desktop app with a custom rendering
  engine. No HTML, CSS, or JavaScript — ever, in the browser or in apps.
- **Apps are separate web applications**, written in Crystal by their
  creators and hosted like current web apps are hosted. A Crystal
  server sends a UI tree to the browser. The browser renders it. The
  browser never runs app code.
- Users **join apps**, like joining a server list. They do not type URLs.
- Mugen's core promises, in order: no URLs, ad/popup free, one single
  account/profile for all apps, no JavaScript bloat, accessibility first.

If a change would require HTML, a URL bar, a third-party script, an
ad/popup node type, or running untrusted code inside the browser,
stop and ask. That is a product change, not an implementation detail.

## 2. The boundary — this is the most important section

### This repo is the browser ONLY
This repo contains only the Rust browser. It contains no apps, no
Crystal code, no app SDK, and no `app-kit/` or `apps/` folders.
Those belonged to an earlier monorepo layout and were removed.

Apps live in **separate repositories**, because realistically they
are made by other people. Example (proposed, not yet on GitHub):
`git@github.com:davidsth/mugen-demo-app.git` — a Crystal demo app
that runs its own local web server.

### The browser owns ONLY:
- Discovery, join, leave/remove of apps
- The joined-app list and switching between apps
- App lifecycle: suspend/resume, snapshot switching
- Auth tokens and the single user profile
- Per-app permissions and per-app storage (isolated by app ID)
- Rendering, layout, input, accessibility

### Each app owns everything inside its window:
- Its own navigation, layout, state, database, and real-time logic
- Its own routes/views — the browser does not impose a channel,
  guild, or page model on it

### Explicitly NOT core concepts:
No guild/channel model, no member list, no presence, no roles,
no DMs, no chat model, no unread/message system in the shell.
The Discord comparison applies ONLY to "a list of joined apps".
Do not reintroduce those concepts as shell features. An app may
implement chat internally; the shell must not know about it.

### Two separate processes, two separate repos:
```
mugen (this repo)     — Rust browser. Ships as a desktop binary.
mugen-demo-app        — Crystal demo app. Ships as a web server.
                        Its own repo, its own shard, its own deployment.
                        NOT part of the browser binary, NOT loaded
                        into the browser process.
```
Never import Crystal code into Rust, never `dlopen` an app, and
never embed an app into the browser binary. The only connection
between browser and app is the protocol in §4, over a network
transport. A crash in an app must not crash the browser.

### Adding an app locally, by IP:port
Like developing a local web app: spin up the app's server, then tell
Mugen its IP and port. The browser accepts this in v0.1 via:

```sh
cargo run -p mugen-shell -- --app 127.0.0.1:4000
cargo run -p mugen-shell -- --app=127.0.0.1:4000
MUGEN_LOCAL_APP=127.0.0.1:4000 cargo run -p mugen-shell
```

The shell logs `http://<endpoint>/manifest` and
`ws://<endpoint>/ws` and adds a sidebar icon. See §9 for the honest
v0.1 limitation: registration is implemented, Snapshot rendering
over WebSocket is not yet.

## 3. Architecture at a glance

```
User
  │
  ▼
browser/crates/shell     winit + softbuffer window (v0.1),
                         joined-app list, click-to-switch,
                         --app IP:port parsing
  │  (Step 3: wgpu renderer + typed layout replace softbuffer)
  ▼
browser/crates/layout    typed constraint layout (Row/Column/Stack/Grid)
  ▼
browser/crates/renderer  wgpu display list, atlas, culling (Step 3)
  ▼
browser/crates/protocol  Snapshot / Patch / Event, validation
  │
  ══ WebSocket ══  (Step 5: shell actually connects and renders)
  │
Separate app repo       Crystal web server, e.g. 127.0.0.1:4000
  GET /manifest, GET /health, WebSocket /ws
  Snapshot on connect, Patch on state change, Events idempotent
```

Identity, not URL (v1, after v0 renders):
- App identity = hash of the publisher's public key.
- Signed manifest: name, icon, version, permissions, endpoints.
  Endpoints are hints only — they can move without changing identity.
- The client pins the key on first verified join and shows a preview
  card (name, publisher, requested permissions) BEFORE first render.
- The published-apps directory distributes manifests; it does not
  issue identity. A compromised directory cannot fake an app without
  its private key. Never commit a private/signing key (see .gitignore).

## 4. Protocol v0 — do not break this casually

Transport: WebSocket. Production encoding is binary (postcard or
FlatBuffers — pick one before the first demo; do NOT use JSON on the
hot path for production). The demo server uses JSON for now, so its
Snapshot/Patch can be inspected with curl and in logs — that is a
deliberate demo simplification, not the production encoding.

- `Hello { protocol_version, app_id, shell_version }` — version
  negotiation on connect. Unknown node types are ignored, never crash.
- `Snapshot { version, root }` — on connect / reconnect / resume.
- `Patch { base_version, upserts, deletes, moves }` — diffs only,
  coalesced to ~one per frame tick (16 ms).
- `Event { event_id, node_id, kind, payload }` — shell -> app,
  idempotent. Retries must not double-apply.
- Separate streams for UI patches vs. live data, so a data flood
  cannot block an input event.

Node types (the complete v0 set — do not add one without a protocol
version bump and a reason in the PR description):
`Box, Text, Image, Scroll, Stack, Input, Button, List, Canvas, Video`

- Layout and style are typed enums/numbers only. No CSS, no string
  style language. If you find yourself parsing a style string, you
  are doing it wrong.
- `Canvas` is a privileged, opt-in capability, not the default API.
  Declarative nodes first, so theming, hit-testing, and accessibility
  stay intact.
- Text: do NOT write a shaper. Use cosmic-text / HarfBuzz underneath.
- Accessibility: expose a semantic tree via accesskit. Accessibility
  first is a core promise, not a polish item. A feature that cannot
  be expressed in the accessibility tree is not done.

Validation, hard, in Rust, before layout (see
`browser/crates/protocol/src/lib.rs`):
max tree depth, max node count, max text length, max image size,
no duplicate node IDs. An app must never be able to DoS or crash
the renderer. Reject, log, and keep the shell alive.

## 5. Commands (macOS + VS Code, the primary dev setup)

Prerequisites, once:
```sh
xcode-select --install
brew install rustup
rustup toolchain install stable
```

Browser (Rust workspace lives under `browser/` — run cargo from there,
not from the repo root):
```sh
cd browser
cargo test -p mugen-protocol
cargo run -p mugen-shell
cargo run -p mugen-shell -- --app 127.0.0.1:4000
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

Demo app (separate repo — Crystal, stdlib only, no external shards):
```sh
cd mugen-demo-app
crystal run src/server.cr
# or: crystal build src/server.cr -o demo-app && ./demo-app
```

CI for this repo runs on macOS, Rust only:
`cargo test -p mugen-protocol --manifest-path browser/Cargo.toml`
and `cargo check --workspace --manifest-path browser/Cargo.toml`.
There is no Crystal job in this repo — Crystal lives in app repos,
each with their own CI if they want one.

## 6. How to work on each side

### Working on the browser (this repo)
1. Change the smallest crate that owns the problem:
   `protocol` for message/validation, `layout` for sizing/placement,
   `renderer` for painting, `shell` for list/lifecycle/local-adding.
2. Keep the public protocol unchanged unless the task explicitly
   says to bump it. If you bump it, update `docs/protocol-v0.md`
   and note that every app repo must mirror the change — do not
   claim app repos are updated until their owners update them.
3. Prefer a failing test in `mugen-protocol` first for validation
   changes. A tree that should be rejected must have a test proving it.
4. Do not add Crystal code, an app, or an app SDK to this repo.
   If a task needs app-side work, it belongs in that app's repo.

### Building a new app (in its own repo, NOT here)
1. Create a new repository for the app (do not add it to this repo).
   Copy the demo-app repo as a template, do not copy browser code.
2. Define the app's UI as a tree (Snapshot/Patch), never HTML.
   Serve `GET /manifest`, `GET /health`, and WebSocket `/ws`.
3. Run the app as its own web server process, e.g. `127.0.0.1:4000`.
   Test it with curl first, then add it to Mugen with
   `--app 127.0.0.1:4000`.
4. The app must work with zero browser changes. If it needs a browser
   change, that is a protocol gap — name it explicitly.
5. Publish: sign a manifest with the publisher key. The browser
   verifies the signature and key before rendering (v1). Never ship
   the private key with the app, in the repo, or in a container image.

### Adding a feature that touches both sides
Land it in this order, in separate repos/commits if possible:
protocol types + validation -> app repo (demo) -> browser
rendering/shell. Never land a browser feature that only one app uses
unless it is a general node/capability with a protocol version.

## 7. Conventions

- Rust: `cargo fmt`, clippy clean, no `unwrap()` on untrusted input
  (protocol data, network data, app trees). `unwrap()` is only for
  programmer errors in tests/dev paths, with a comment if non-obvious.
- No new dependencies without a reason in the PR: what it replaces,
  what it costs on macOS, and whether it is needed for v0 or can wait.
- Keep diffs small and repo-scoped: a browser PR should not also
  rewrite an app, and vice versa, unless the protocol changed.
- Documentation lives with the code it describes: protocol changes
  update `docs/protocol-v0.md`, browser changes update
  `browser/README.md`.

## 8. What NOT to do (v1)

- No HTML/CSS/JS anywhere. No WebView, no embedded browser engine.
- No general address bar, no arbitrary URL entry in the browser UI.
  Local IP:port adding is a developer join path, not a URL bar:
  it registers a known local server, it does not browse the web.
- No dlopen/native plugin loading of apps. No WASM runtime for apps.
- No hand-written GPU backend, no hand-written text shaper.
- No ads, popups, trackers, or third-party scripts — there is no
  node type for them, and there must not become one by accident
  (e.g. a generic "embed" node that can load arbitrary content).
- No cross-app data access. Storage, tokens, and profile data are
  keyed by app ID, and an app can only reach its own data via the
  shell's storage broker.
- No committing keys, tokens, `.env` files, or `Cargo.lock` removal.
  (Cargo.lock IS committed — this ships a binary.)
- No adding apps, app SDKs, or Crystal code back into this repo.

## 9. Current status — be honest about it in every PR description

- v0.1 browser: real winit + softbuffer window, joined-app sidebar,
  click-to-switch, local-app registration via `--app` / env var.
  Protocol crate is real and tested. Layout/renderer are placeholders
  for Step 3 (wgpu), not yet wired into the shell.
- The shell does NOT yet fetch/render a Snapshot over WebSocket.
  A local app's pane is a placeholder layout in v0.1. The next step
  (Step 5) is to connect `ws://<ip:port>/ws` and render the Snapshot.
  Do not describe v0.1 as rendering live app data.
- The demo app repo is a separate, stdlib-only Crystal web server
  (manifest, health, WebSocket Snapshot/Patch for a task list).
  It is a local development server, not a published app.
- Nothing here is a finished browser yet. Do not describe it as one
  in READMEs, landing pages, or PR titles. Say "early development" /
  "planned" where a feature is not yet built.
- First builds are verified on macOS (the maintainer's machine).
  This repo's CI is macOS-only, Rust-only, for that reason.
- This workspace's VM has no Rust/Crystal toolchain, so no build
  was run here. macOS verification is the source of truth.

## 10. If you are unsure

Ask, with the specific tradeoff, rather than guessing and landing a
large change. In particular, ask before: changing the protocol,
adding a node type, adding a permission, changing identity/key
handling, adding a dependency, or adding anything app-related back
into this repo. A short question now is cheaper than a protocol
break later.
