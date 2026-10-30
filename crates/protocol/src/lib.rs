//! Mugen protocol v0 — types and validation only.
//!
//! Transport for v0: WebSocket, binary encoding (postcard or
//! FlatBuffers — pick one before the first demo; do NOT use JSON on
//! the hot path). This crate deliberately has no external dependencies
//! so `cargo test -p mugen-protocol` works on a fresh macOS checkout.
//!
//! Messages:
//! - `Snapshot { version, root }` — on connect / reconnect / resume
//!   when the patch gap is too old.
//! - `Patch { base_version, upserts, deletes, moves }` — diffs only.
//! - `Event { event_id, node_id, kind, payload }` — shell -> app,
//!   idempotent: retries must not double-apply.
//!
//! Validation is hard, in Rust, before layout: an app must never be
//! able to DoS or crash the renderer.

/// Current protocol version. Negotiated on connect (`Hello`).
pub const PROTOCOL_VERSION: u32 = 0;

/// Hard validation limits (v0). Tune only with a protocol version bump.
pub const MAX_TREE_DEPTH: usize = 64;
pub const MAX_NODE_COUNT: usize = 10_000;
pub const MAX_TEXT_LEN: usize = 100_000;
pub const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;

/// The 10 typed node kinds. No HTML, no CSS, no string style language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Box,
    Text,
    Image,
    Scroll,
    Stack,
    Input,
    Button,
    List,
    Canvas, // privileged, opt-in capability — not the default API
    Video,
}

/// Typed layout only: Row, Column, Stack, Grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutKind {
    Row,
    Column,
    Stack,
    Grid,
}

/// A node in the UI tree. `id` is unique within a tree version.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: u64,
    pub kind: NodeKind,
    /// For Text nodes. Empty for other kinds in v0.
    pub text: String,
    pub layout: LayoutKind,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(id: u64, kind: NodeKind) -> Self {
        Self {
            id,
            kind,
            text: String::new(),
            layout: LayoutKind::Column,
            children: Vec::new(),
        }
    }

    pub fn text(id: u64, text: impl Into<String>) -> Self {
        let mut node = Self::new(id, NodeKind::Text);
        node.text = text.into();
        node
    }

    pub fn count(&self) -> usize {
        1 + self.children.iter().map(Node::count).sum::<usize>()
    }

    pub fn depth(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(Node::depth)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub version: u64,
    pub root: Node,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub base_version: u64,
    pub version: u64,
    pub upserts: Vec<Node>,
    pub deletes: Vec<u64>,
    pub moves: Vec<(u64, u64)>, // (node_id, new_parent_id)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub event_id: u64,
    pub node_id: u64,
    pub kind: String, // e.g. "click", "submit", "change" — closed set in v1
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    TooDeep,
    TooManyNodes,
    TextTooLong,
    DuplicateId(u64),
}

/// Validate a snapshot before it ever reaches layout or painting.
/// Unknown node kinds are ignored by the decoder (not here — this
/// crate only models known kinds); they must never crash the shell.
pub fn validate_snapshot(snapshot: &Snapshot) -> Result<(), ValidationError> {
    if snapshot.root.depth() > MAX_TREE_DEPTH {
        return Err(ValidationError::TooDeep);
    }
    if snapshot.root.count() > MAX_NODE_COUNT {
        return Err(ValidationError::TooManyNodes);
    }
    let mut seen = Vec::new();
    validate_node(&snapshot.root, &mut seen)?;
    Ok(())
}

fn validate_node(node: &Node, seen: &mut Vec<u64>) -> Result<(), ValidationError> {
    if node.text.len() > MAX_TEXT_LEN {
        return Err(ValidationError::TextTooLong);
    }
    if seen.contains(&node.id) {
        return Err(ValidationError::DuplicateId(node.id));
    }
    seen.push(node.id);
    for child in &node.children {
        validate_node(child, seen)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> Node {
        let mut root = Node::new(1, NodeKind::Box);
        root.layout = LayoutKind::Column;
        root.children.push(Node::text(2, "Tasks"));
        root.children.push(Node::new(3, NodeKind::Button));
        root
    }

    #[test]
    fn valid_snapshot_passes() {
        let snap = Snapshot {
            version: 1,
            root: sample_tree(),
        };
        assert!(validate_snapshot(&snap).is_ok());
        assert_eq!(snap.root.count(), 3);
    }

    #[test]
    fn duplicate_id_is_rejected() {
        let mut root = sample_tree();
        root.children.push(Node::text(2, "duplicate"));
        let snap = Snapshot {
            version: 1,
            root,
        };
        assert_eq!(
            validate_snapshot(&snap),
            Err(ValidationError::DuplicateId(2))
        );
    }
}
