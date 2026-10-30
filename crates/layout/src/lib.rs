//! Step 3 placeholder.
//!
//! Implement here: two-pass constraint layout (measure, then place)
//! for Row / Column / Stack / Grid, with caching — only dirty nodes
//! re-layout. Constraints, padding, gap, alignment, grow, shrink,
//! min/max size are typed values, never CSS strings.
//!
//! Text measurement will come from the renderer (cosmic-text), via a
//! trait, so layout stays testable without a GPU.

use mugen_protocol::Node;

/// A laid-out node: the input node plus its computed box.
/// Filled in during Step 3.
#[derive(Debug, Clone)]
pub struct LaidOut {
    pub node_id: u64,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn layout(_root: &Node, _width: f32, _height: f32) -> Vec<LaidOut> {
    // TODO (Step 3): implement measure + place + dirty tracking.
    Vec::new()
}
