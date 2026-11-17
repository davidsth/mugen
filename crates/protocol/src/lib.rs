//! Mugen protocol v1: typed UI trees, patches, and interaction events.
//! Transport and serialization are intentionally kept outside this crate.

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_TREE_DEPTH: usize = 64;
pub const MAX_NODE_COUNT: usize = 10_000;
pub const MAX_TEXT_LEN: usize = 100_000;
pub const MAX_EVENT_TEXT_LEN: usize = 16_384;
pub const MAX_PATCH_OPERATIONS: usize = 10_000;
pub const MAX_HANDLER_NAME_LEN: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind { Box, Text, Image, Scroll, Stack, Input, Button, List, Canvas, Video }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis { Horizontal, Vertical }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align { Start, Center, End, Stretch }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justify { Start, Center, End, SpaceBetween, SpaceAround }
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length { Auto, Px(f32), Percent(f32), Fill }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutKind { Row, Column, Stack, Grid }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind { Click, Change, Submit, Focus, Blur, KeyDown, KeyUp, Scroll, Activate }

#[derive(Debug, Clone, PartialEq)]
pub enum EventValue {
    None, Bool(bool), Number(f64), Text(String),
    Key { key: String, modifiers: u8 },
    Scroll { x: f64, y: f64 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventErrorCode { UnknownNode, UnknownHandler, InvalidValue, StaleVersion, PermissionDenied }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets { pub top: f32, pub right: f32, pub bottom: f32, pub left: f32 }
impl Insets { pub const ZERO: Self = Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 }; }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub foreground: Option<Color>, pub background: Option<Color>, pub border: Option<Color>,
    pub border_width: f32, pub radius: f32, pub opacity: f32,
}
impl Default for Style {
    fn default() -> Self {
        Self { foreground: None, background: None, border: None, border_width: 0.0, radius: 0.0, opacity: 1.0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub kind: LayoutKind, pub axis: Axis, pub align: Align, pub justify: Justify,
    pub gap: f32, pub padding: Insets, pub width: Length, pub height: Length,
    pub min_width: Option<f32>, pub min_height: Option<f32>,
    pub max_width: Option<f32>, pub max_height: Option<f32>,
    pub grow: f32, pub shrink: f32,
}
impl Default for Layout {
    fn default() -> Self {
        Self { kind: LayoutKind::Column, axis: Axis::Vertical, align: Align::Stretch,
            justify: Justify::Start, gap: 0.0, padding: Insets::ZERO,
            width: Length::Auto, height: Length::Auto, min_width: None, min_height: None,
            max_width: None, max_height: None, grow: 0.0, shrink: 1.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Props {
    pub placeholder: Option<String>, pub value: Option<String>, pub resource: Option<String>,
    pub accessible_label: Option<String>, pub disabled: bool, pub checked: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventBinding { pub event: EventKind, pub handler: String }

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: u64, pub kind: NodeKind, pub text: String, pub layout: Layout,
    pub style: Style, pub props: Props, pub events: Vec<EventBinding>, pub children: Vec<Node>,
}
impl Node {
    pub fn new(id: u64, kind: NodeKind) -> Self {
        Self { id, kind, text: String::new(), layout: Layout::default(), style: Style::default(),
            props: Props::default(), events: Vec::new(), children: Vec::new() }
    }
    pub fn text(id: u64, text: impl Into<String>) -> Self {
        let mut node = Self::new(id, NodeKind::Text); node.text = text.into(); node
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot { pub version: u64, pub root: Node }
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub base_version: u64, pub version: u64, pub upserts: Vec<Node>,
    pub deletes: Vec<u64>, pub moves: Vec<NodeMove>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeMove { pub node_id: u64, pub new_parent_id: u64 }
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub event_id: u64, pub tree_version: u64, pub node_id: u64,
    pub kind: EventKind, pub value: EventValue,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Hello { protocol_version: u32, app_id: String, shell_version: String },
    Event(Event), ResyncRequest { have_version: u64 }, EventAck { event_id: u64 },
}
#[derive(Debug, Clone, PartialEq)]
pub enum ServerMessage {
    Welcome { protocol_version: u32, app_name: String }, Snapshot(Snapshot), Patch(Patch),
    EventError { event_id: u64, code: EventErrorCode, message: String },
    Ping { nonce: u64 }, Pong { nonce: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    ZeroNodeId, TooDeep, TooManyNodes, TooManyOperations, TextTooLong,
    EventTextTooLong, DuplicateId(u64), InvalidHandler, InvalidNumber,
    InvalidResourceScheme, InvalidVersion, DuplicateOperation(u64),
}

pub fn validate_snapshot(snapshot: &Snapshot) -> Result<(), ValidationError> {
    let mut seen = Vec::new();
    let mut count = 0;
    validate_node(&snapshot.root, 1, &mut count, &mut seen)
}

fn validate_node(node: &Node, depth: usize, count: &mut usize, seen: &mut Vec<u64>) -> Result<(), ValidationError> {
    if depth > MAX_TREE_DEPTH { return Err(ValidationError::TooDeep); }
    *count += 1;
    if *count > MAX_NODE_COUNT { return Err(ValidationError::TooManyNodes); }
    if node.id == 0 { return Err(ValidationError::ZeroNodeId); }
    if seen.contains(&node.id) { return Err(ValidationError::DuplicateId(node.id)); }
    seen.push(node.id);
    if node.text.len() > MAX_TEXT_LEN
        || node.props.placeholder.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.props.value.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.props.accessible_label.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN) {
        return Err(ValidationError::TextTooLong);
    }
    if let Some(uri) = &node.props.resource {
        if !(uri.starts_with("app://") || uri.starts_with("https://")) {
            return Err(ValidationError::InvalidResourceScheme);
        }
    }
    validate_layout(&node.layout)?;
    validate_style(&node.style)?;
    for binding in &node.events {
        if !valid_handler(&binding.handler) { return Err(ValidationError::InvalidHandler); }
    }
    for child in &node.children { validate_node(child, depth + 1, count, seen)?; }
    Ok(())
}

fn validate_layout(layout: &Layout) -> Result<(), ValidationError> {
    let values = [layout.gap, layout.padding.top, layout.padding.right, layout.padding.bottom,
        layout.padding.left, layout.grow, layout.shrink];
    if values.iter().any(|v| !v.is_finite() || *v < 0.0) { return Err(ValidationError::InvalidNumber); }
    for value in [layout.min_width, layout.min_height, layout.max_width, layout.max_height].into_iter().flatten() {
        if !value.is_finite() || value < 0.0 { return Err(ValidationError::InvalidNumber); }
    }
    for length in [layout.width, layout.height] {
        match length {
            Length::Px(v) if !v.is_finite() || v < 0.0 => return Err(ValidationError::InvalidNumber),
            Length::Percent(v) if !v.is_finite() || !(0.0..=100.0).contains(&v) => return Err(ValidationError::InvalidNumber),
            _ => {}
        }
    }
    Ok(())
}

fn validate_style(style: &Style) -> Result<(), ValidationError> {
    if !style.border_width.is_finite() || style.border_width < 0.0
        || !style.radius.is_finite() || style.radius < 0.0
        || !style.opacity.is_finite() || !(0.0..=1.0).contains(&style.opacity) {
        return Err(ValidationError::InvalidNumber);
    }
    Ok(())
}

fn valid_handler(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_HANDLER_NAME_LEN
        && value.bytes().enumerate().all(|(i, b)| {
            b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || (i > 0 && b == b'.')
        })
}

pub fn validate_patch(patch: &Patch) -> Result<(), ValidationError> {
    if patch.version <= patch.base_version { return Err(ValidationError::InvalidVersion); }
    if patch.upserts.len() + patch.deletes.len() + patch.moves.len() > MAX_PATCH_OPERATIONS {
        return Err(ValidationError::TooManyOperations);
    }
    let mut ids = Vec::new();
    let mut count = 0;
    for node in &patch.upserts { validate_node(node, 1, &mut count, &mut ids)?; }
    let mut touched = Vec::new();
    for id in patch.deletes.iter().copied().chain(patch.moves.iter().map(|m| m.node_id)) {
        if id == 0 { return Err(ValidationError::ZeroNodeId); }
        if touched.contains(&id) || ids.contains(&id) { return Err(ValidationError::DuplicateOperation(id)); }
        touched.push(id);
    }
    for movement in &patch.moves {
        if movement.new_parent_id == 0 { return Err(ValidationError::ZeroNodeId); }
    }
    Ok(())
}

pub fn validate_event(event: &Event) -> Result<(), ValidationError> {
    if event.event_id == 0 || event.node_id == 0 { return Err(ValidationError::ZeroNodeId); }
    match &event.value {
        EventValue::Text(value) if value.len() > MAX_EVENT_TEXT_LEN => return Err(ValidationError::EventTextTooLong),
        EventValue::Number(value) if !value.is_finite() => return Err(ValidationError::InvalidNumber),
        EventValue::Key { key, .. } if key.len() > 256 => return Err(ValidationError::EventTextTooLong),
        EventValue::Scroll { x, y } if !x.is_finite() || !y.is_finite() => return Err(ValidationError::InvalidNumber),
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_tree() -> Node {
        let mut root = Node::new(1, NodeKind::Box);
        root.children.push(Node::text(2, "Tasks"));
        let mut button = Node::new(3, NodeKind::Button);
        button.events.push(EventBinding { event: EventKind::Click, handler: "tasks.add".into() });
        root.children.push(button);
        root
    }

    #[test] fn valid_snapshot_passes() {
        assert!(validate_snapshot(&Snapshot { version: 1, root: sample_tree() }).is_ok());
    }
    #[test] fn duplicate_and_zero_ids_are_rejected() {
        let mut root = sample_tree(); root.children.push(Node::text(2, "duplicate"));
        assert_eq!(validate_snapshot(&Snapshot { version: 1, root }), Err(ValidationError::DuplicateId(2)));
        assert_eq!(validate_snapshot(&Snapshot { version: 1, root: Node::new(0, NodeKind::Box) }), Err(ValidationError::ZeroNodeId));
    }
    #[test] fn unsafe_resource_schemes_are_rejected() {
        let mut root = sample_tree(); root.props.resource = Some("javascript:alert(1)".into());
        assert_eq!(validate_snapshot(&Snapshot { version: 1, root }), Err(ValidationError::InvalidResourceScheme));
    }
    #[test] fn handler_names_are_symbolic_not_code() {
        let mut root = sample_tree(); root.children[1].events[0].handler = "tasks.add();panic".into();
        assert_eq!(validate_snapshot(&Snapshot { version: 1, root }), Err(ValidationError::InvalidHandler));
    }
    #[test] fn patches_require_monotonic_versions() {
        let bad = Patch { base_version: 4, version: 4, upserts: vec![], deletes: vec![], moves: vec![] };
        assert_eq!(validate_patch(&bad), Err(ValidationError::InvalidVersion));
        let good = Patch { base_version: 4, version: 5, upserts: vec![Node::new(8, NodeKind::Text)], deletes: vec![], moves: vec![] };
        assert!(validate_patch(&good).is_ok());
    }
    #[test] fn event_payload_is_bounded() {
        let event = Event { event_id: 1, tree_version: 1, node_id: 2, kind: EventKind::Change,
            value: EventValue::Text("x".repeat(MAX_EVENT_TEXT_LEN + 1)) };
        assert_eq!(validate_event(&event), Err(ValidationError::EventTextTooLong));
    }
}
