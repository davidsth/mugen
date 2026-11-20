# Layout engine milestone

The initial `mugen-layout` crate provides a deterministic, renderer-independent layout pass for protocol `Node` trees.

## Implemented

- Preorder geometry output keyed by node ID.
- Row and Column placement with gaps and padding.
- Stack overlay placement.
- Typed `Auto`, `Px`, `Percent`, and `Fill` sizing.
- Min/max clamping and basic grow distribution.
- Start/center/end/stretch cross-axis alignment.
- Start/center/end and space-between/space-around main-axis justification.
- Conservative intrinsic sizing for text (8 logical px per character, 20 px line height).
- Invalid viewport values sanitized to zero.
- Unit tests for row, column, padding, gaps, stack, and invalid viewport.

## Deliberate limitations

This is a foundational pass, not a production layout system. Grid currently follows a horizontal flow and needs an explicit grid-track implementation. Shrink is not yet applied. Text measurement is approximate and must be replaced with a renderer-provided measurement interface. Overflow, scrolling, baseline alignment, and layout caching/dirty-node invalidation remain future work. Layout is not yet wired into the shell renderer.

## Next

1. Correct sizing and overflow semantics, especially fill/grow/shrink and cross-axis constraints.
2. Implement explicit Grid tracks.
3. Add text measurement abstraction and golden geometry tests.
4. Integrate layout output into the renderer.
