# Crystal app SDK contract (separate repository)

This is an interface specification for a future Crystal shard. **Do not add Crystal source or an app SDK to the Mugen browser repository.** The browser and each app remain separate processes and repositories.

## Design goals

- Crystal authors define typed view trees and handlers without HTML, CSS, or JavaScript.
- Apps own routes, domain state, database access, authorization, and persistence.
- The SDK mirrors protocol v1 and has no dependency on Rust internals.
- Handler names are explicit symbolic identifiers; no reflection-based arbitrary method invocation.
- A local development server can expose `GET /manifest`, `GET /health`, and WebSocket `/ws`.

## Proposed API surface

The following is illustrative pseudocode, not a committed or compile-verified Crystal API:

```crystal
class TasksView
  include Mugen::View

  def build(state : TasksState) : Mugen::Node
    column(id: 1, gap: 12) do
      text(id: 2, "Tasks")
      input(id: 3, value: state.draft, on_change: "tasks.draft")
      button(id: 4, "Add", on_click: "tasks.add")
    end
  end
end

session.on("tasks.add") do |event, state|
  state.add_task(event.text_value)
end
```

The eventual shard should provide typed node/layout/style builders, typed event values, a HandlerRegistry with explicit name-to-handler registration, a per-client Session, snapshot generation, monotonically versioned patch generation, a stable-ID tree reconciler, a codec abstraction (initially JSON), bounded event deduplication, and reconnect snapshot behavior.

## State and patch generation

The app's domain model is authoritative. A view builder projects domain state into a tree. The reconciler compares the previous and next trees and emits a patch with a matching base version. If a safe diff cannot be generated, send a full snapshot. Do not expose database objects or arbitrary server internals as UI properties.

## Milestones for the separate SDK

1. Rust/Crystal golden vectors for every protocol message and enum.
2. Crystal protocol types and JSON codec.
3. Local stdlib HTTP/WebSocket server with manifest and health routes.
4. Handler registry, event deduplication, and session lifecycle.
5. Tree builder and snapshot serializer.
6. Tree reconciler and patch serializer.
7. Example task-list app in its own repository.

The browser repo should not claim this SDK exists until that separate repository has been implemented and tested.
