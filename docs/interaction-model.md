# Mugen interaction model

This document defines how an app UI becomes interactive while keeping app code outside the Rust browser process.

## Responsibility split

**Browser (Rust):** pointer hit testing, hover/pressed states, focus and tab order, keyboard dispatch, text editing, selection, scrolling, accessibility tree, local animation, and native rendering.

**App server (Crystal, separate process):** routes, domain state, validation, authorization, persistence, business rules, and decisions that change the app's authoritative UI tree.

## Event round trip

1. Browser receives a pointer or keyboard action.
2. Browser updates local transient state immediately (pressed/focus/selection feedback).
3. Browser resolves the target node's symbolic event binding.
4. Browser sends an Event with unique event ID, node ID, event kind, typed value, and observed tree version.
5. Crystal validates the event against the current app/session and resolves the symbolic handler through an allowlist.
6. Crystal applies domain logic and emits an authoritative versioned Patch (or a structured event error).
7. Browser verifies base version and limits, applies the patch atomically, updates accessibility semantics, and renders.
8. Browser acknowledges/deduplicates event IDs. A retry must not repeat a committed side effect.

## Input behavior

- Text input is edited locally for responsiveness; changes are sent as typed Change events, with explicit submit/commit semantics where appropriate.
- Buttons expose Click and keyboard Activate behavior.
- Focus and blur are local first; apps may observe them but must not be required to manage focus.
- Scroll is local and can be reported to the app only when the app has a semantic reason to observe it.
- Disabled controls do not dispatch activation events.
- Unknown handlers and stale tree versions produce a bounded error and do not crash the shell.

## Ordering and backpressure

Use a reliable ordered control channel for events and acknowledgements. Visual patches may be coalesced, but event delivery must not be blocked by a burst of non-critical visual updates. Bound queues and payload sizes. When the client falls behind, stop applying incremental patches and request a fresh snapshot.

## Accessibility-first component contract (WAI-ARIA)

Accessibility is a design constraint for every component from its first commit. Component APIs and examples must include their semantic behavior, not bolt it on later.

- Model WAI-ARIA roles, accessible names/descriptions, states, properties, relationships, and live-region semantics using typed protocol fields. Follow the current WAI-ARIA specification and APG interaction patterns.
- Prefer each component's correct built-in semantic role and behavior. Authors may supply additional semantics when needed, but cannot create invalid or contradictory role/state combinations.
- Provide validated relationships (for example, labelled-by, described-by, controls, and owns) with stable node references. Broken references must be rejected or safely degraded.
- Map semantics through AccessKit to each platform's native accessibility API. Do not introduce a DOM, WebView, or HTML layer to achieve ARIA compatibility.
- Every action must be available through keyboard and assistive technology. Define tab order, arrow-key behavior for composite widgets, activation keys, Escape behavior, focus visibility, and focus restoration according to the applicable APG pattern.
- Ensure accessible names for controls, alternatives for meaningful images, announcements for dynamic status/errors, and correct checked/selected/expanded/disabled/value state.
- Respect user preferences such as reduced motion and text scaling. Never use color or animation as the only carrier of information.

## Focus, accessibility, and keyboard

Stable node IDs preserve focus across patches where possible. If a focused node is deleted or disabled, move focus to a deterministic nearby focusable node or the app root. Keyboard navigation follows native platform conventions and the applicable APG pattern. The semantic tree mirrors the rendered tree and exposes names, roles, values, relationships, states, and supported actions. Accessibility semantics and focus must remain correct after every patch.

## Optimistic state

Only transient presentation may be optimistic. Persistent or security-sensitive state is always server-authoritative. On rejection or conflicting patch, restore the last authoritative value and surface a concise error through the app's UI.
