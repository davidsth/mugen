# Mugen protocol v1

Status: **contract draft for the v0.2 milestone**. This document describes the intended wire contract; it does not claim that the shell's WebSocket transport or native renderer is implemented.

## Goals and boundaries

- Rust browser and Crystal app server remain separate processes and repositories.
- The browser owns rendering, layout, input, accessibility, app discovery/lifecycle, permissions, and isolated per-app storage.
- Each app owns its routes, view tree, state, persistence, and business logic.
- No HTML, CSS, JavaScript, WebView, arbitrary URL navigation, embedded Crystal runtime, WASM, or native app-code loading.
- Protocol types are transport-independent. The current protocol crate has no serialization dependency.

## Connection lifecycle

1. The client opens the app's configured WebSocket endpoint.
2. Client sends `Hello { protocol_version, app_id, shell_version }`.
3. Server replies with `Welcome { protocol_version, app_name }`; incompatible protocol versions close with a clear error.
4. Server sends a full `Snapshot { version, root }`.
5. Server sends incremental `Patch { base_version, version, upserts, deletes, moves }` messages.
6. Client sends `Event` messages for user interactions and acknowledges accepted event IDs.
7. On a version gap, reconnect, or failed patch application, client requests a full snapshot using `ResyncRequest`.

The initial implementation may serialize the same typed model as JSON for local development. Do not select postcard, FlatBuffers, or another binary codec until Rust and Crystal golden vectors exist and are checked in.

## Tree model

Every node has a stable, non-zero ID, a closed `NodeKind`, typed `Layout` and `Style`, `Props`, event bindings, and children. The v1 node set is Box, Text, Image, Scroll, Stack, Input, Button, List, Canvas, and Video.

Layout uses typed enums and numeric values only: Row/Column/Stack/Grid, axis, alignment, justification, gap, insets, dimensions, min/max dimensions, grow, and shrink. Style is typed color, border, radius, and opacity. There is no free-form style string and no CSS parser.

Canvas and Video are capability-gated. Their presence in the enum does not grant an app permission to use them. The shell must check granted capabilities before allocating resources or rendering privileged content.

## Events and state ownership

Event kinds are a closed enum: Click, Change, Submit, Focus, Blur, KeyDown, KeyUp, Scroll, Activate. Values are typed (none, boolean, number, text, key/modifiers, scroll offsets). An event carries a unique event ID, the tree version the user interacted with, and the target node ID.

The browser handles immediate local interaction: hover, pressed state, focus traversal, pointer capture, scroll physics, keyboard routing, and text editing. It sends semantic events to the app for business decisions. The app remains authoritative for persistent state and returns patches. Optimistic input is allowed only for transient presentation and must reconcile to the next authoritative patch.

Handlers are symbolic names such as `tasks.add`, never code, expressions, or arbitrary method paths. The app resolves them through its own allowlisted handler registry.

## Patch semantics

- `base_version` must equal the client's current tree version.
- `version` must be greater than `base_version`.
- Apply each patch atomically; never render a partially applied patch.
- Upserts replace or insert complete subtrees; deletes remove IDs; moves change parent while preserving node identity.
- The future tree-store must reject duplicate IDs, conflicting operations, unknown parents, cycles, and references to missing nodes.
- If a patch cannot be applied, keep the last good tree and request a snapshot.

The current protocol crate validates message shape and resource bounds. Full patch application and graph/reference validation belong in the tree-store implementation and are not claimed complete by this milestone.

## Limits and validation

The Rust boundary validates before layout or rendering:
- maximum depth: 64
- maximum nodes per tree: 10,000
- maximum text: 100,000 bytes per text-bearing field
- maximum event text: 16 KiB
- maximum patch operations: 10,000
- non-zero, unique node IDs
- finite, non-negative layout/style numbers and opacity in [0, 1]
- symbolic handler names only
- resource schemes limited to `app://` and `https://`

Scheme validation is not authorization. The resource broker must additionally enforce app origin, granted permissions, redirect policy, byte limits, and per-app isolation. Unknown node kinds from a future decoder should be handled as unsupported content, not as a panic.

## Delivery and reliability

Input events should use a reliable, ordered control path distinct from coalescible visual updates. Event IDs support deduplication; the app should retain a bounded recent-ID window and make handlers idempotent. Patches can be coalesced to roughly one per frame (16 ms), but must not starve input or accessibility updates. Reconnect starts with version negotiation and a fresh snapshot unless bounded replay is explicitly added later.

## Accessibility

Every interactive node must expose an accessible name/role/state and keyboard behavior. The native renderer should produce an accesskit semantic tree. A control is not complete until it can be operated without a pointer.

## Explicit non-goals for v0.2

No live WebSocket connection in the shell, no complete renderer/layout engine, no Crystal SDK implementation in this repository, no production binary codec, no app identity/signature system, and no working demo app are delivered by this protocol-specification milestone.
