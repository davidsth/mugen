# Mugen protocol v0 — summary

Full draft (Oct 26, 2026) lives in the working notes; this is the
version that ships with the repo skeleton.

## Boundary

Shell owns only: joined-app list, discovery, join/leave, switching and
lifecycle, auth tokens, per-app permissions/storage.

Each app owns everything inside its window.

Explicitly NOT core concepts: guild/channel, member list, presence,
roles, DMs, chat model, unread/message system.

## Messages

- `Hello { protocol_version, app_id, shell_version }` — version
  negotiation on connect. Unknown node types are ignored, never crash.
- `Snapshot { version, root_node }` — on connect / reconnect / resume.
- `Patch { base_version, upserts, deletes, moves }` — diffs only,
  coalesced to one patch per frame tick (~16 ms).
- `Event { event_id, node_id, kind, payload }` — shell → app,
  idempotent.
- Separate streams for UI patches vs. live data, so a data flood cannot
  block an input event.

## Nodes and style

`Box, Text, Image, Scroll, Stack, Input, Button, List, Canvas, Video`

Layout: typed `Row, Column, Stack, Grid` with padding, gap, alignment,
grow, shrink, min/max size. Style: typed tokens, not strings, not CSS.

Canvas is a privileged, opt-in capability — declarative nodes first,
so theming, hit-testing, and accessibility stay intact.

## Identity (v1, after v0 renders)

App identity = publisher public key hash. Signed manifest lists name,
icon, version, permissions, and current endpoints (hints only).
The client pins the key on first verified join and refuses a server
that cannot prove it. The published-apps directory distributes
manifests; it does not issue identity. A compromised directory still
cannot fake an app without its private key.

## Switching / lifecycle

Keep last tree + bitmap snapshot per joined app. Switching shows the
snapshot instantly, then applies missed patches. Background apps are
suspended: no layout, no painting.
