//! Mugen protocol v1: typed UI trees, patches, and interaction events.
//! Transport and serialization are intentionally kept outside this crate.

use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityRole {
    Alert,
    AlertDialog,
    Application,
    Article,
    Banner,
    Blockquote,
    Button,
    Caption,
    Cell,
    Checkbox,
    Code,
    ColumnHeader,
    Combobox,
    Complementary,
    ContentInfo,
    Definition,
    Deletion,
    Dialog,
    /// Deprecated in WAI-ARIA 1.2; retained for compatibility.
    Directory,
    Document,
    Emphasis,
    Feed,
    Figure,
    Form,
    Generic,
    Grid,
    GridCell,
    Group,
    Heading,
    Image,
    Insertion,
    Link,
    List,
    ListBox,
    ListItem,
    Log,
    Main,
    Marquee,
    Math,
    Menu,
    MenuBar,
    MenuItem,
    MenuItemCheckbox,
    MenuItemRadio,
    Meter,
    Navigation,
    None,
    Note,
    Option,
    Paragraph,
    Presentation,
    ProgressBar,
    Radio,
    RadioGroup,
    Region,
    Row,
    RowGroup,
    RowHeader,
    Scrollbar,
    Search,
    Searchbox,
    Separator,
    Slider,
    SpinButton,
    Status,
    Strong,
    Subscript,
    Superscript,
    Switch,
    Tab,
    Table,
    TabList,
    TabPanel,
    Term,
    Textbox,
    Time,
    Timer,
    Toolbar,
    Tooltip,
    Tree,
    TreeGrid,
    TreeItem,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckedState { False, True, Mixed }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressedState { False, True, Mixed }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidState { False, True, Grammar, Spelling }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentState { False, True, Page, Step, Location, Date, Time }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind { Menu, ListBox, Tree, Grid, Dialog }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivePoliteness { Off, Polite, Assertive }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveRelevant { Additions, Removals, Text, All }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoComplete { Inline, List, Both }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortState { None, Ascending, Descending, Other }

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccessibilityStates {
    pub disabled: bool,
    pub expanded: Option<bool>,
    pub selected: Option<bool>,
    pub checked: Option<CheckedState>,
    pub pressed: Option<PressedState>,
    pub required: bool,
    pub read_only: bool,
    pub busy: bool,
    pub invalid: Option<InvalidState>,
    pub modal: bool,
    pub multiline: Option<bool>,
    pub multiselectable: Option<bool>,
    pub current: Option<CurrentState>,
    pub has_popup: Option<PopupKind>,
    pub orientation: Option<Axis>,
    pub value_min: Option<f64>,
    pub value_max: Option<f64>,
    pub value_now: Option<f64>,
    pub value_text: Option<String>,
    pub level: Option<u32>,
    pub position_in_set: Option<u32>,
    pub set_size: Option<u32>,
    pub sort: Option<SortState>,
    pub autocomplete: Option<AutoComplete>,
    pub row_count: Option<i32>,
    pub column_count: Option<i32>,
    pub row_index: Option<u32>,
    pub column_index: Option<u32>,
    pub row_span: Option<u32>,
    pub column_span: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccessibilityRelationships {
    pub labelled_by: Vec<u64>,
    pub described_by: Vec<u64>,
    pub controls: Vec<u64>,
    pub owns: Vec<u64>,
    pub details: Vec<u64>,
    pub flow_to: Vec<u64>,
    pub headers: Vec<u64>,
    pub active_descendant: Option<u64>,
    pub error_message: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveRegion {
    pub politeness: LivePoliteness,
    pub atomic: bool,
    pub relevant: Vec<LiveRelevant>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Accessibility {
    /// None means use the component's native default role.
    pub role: Option<AccessibilityRole>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub role_description: Option<String>,
    pub hidden: bool,
    pub key_shortcuts: Vec<String>,
    pub states: AccessibilityStates,
    pub relationships: AccessibilityRelationships,
    pub live_region: Option<LiveRegion>,
}

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
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventBinding { pub event: EventKind, pub handler: String }

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: u64, pub kind: NodeKind, pub text: String, pub layout: Layout,
    pub style: Style, pub props: Props, pub accessibility: Accessibility,
    pub events: Vec<EventBinding>, pub children: Vec<Node>,
}
impl Node {
    pub fn new(id: u64, kind: NodeKind) -> Self {
        Self { id, kind, text: String::new(), layout: Layout::default(), style: Style::default(),
            props: Props::default(), accessibility: Accessibility::default(),
            events: Vec::new(), children: Vec::new() }
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
    EmptyAccessibilityName, EmptyAccessibilityRoleDescription, NameProhibitedForRole,
    UnknownAccessibilityReference(u64), MultipleAccessibilityOwners(u64),
    AccessibilityOwnershipCycle(u64), InvalidActiveDescendant(u64),
    InvalidAccessibilityState, InvalidAccessibilityRange, InvalidLiveRegionConfiguration,
}

pub fn validate_snapshot(snapshot: &Snapshot) -> Result<(), ValidationError> {
    let mut seen = HashSet::new();
    let mut count = 0;
    validate_node(&snapshot.root, 1, &mut count, &mut seen)?;
    validate_accessibility_relationships(&snapshot.root, &seen)
}

fn validate_node(node: &Node, depth: usize, count: &mut usize, seen: &mut HashSet<u64>) -> Result<(), ValidationError> {
    if depth > MAX_TREE_DEPTH { return Err(ValidationError::TooDeep); }
    *count += 1;
    if *count > MAX_NODE_COUNT { return Err(ValidationError::TooManyNodes); }
    if node.id == 0 { return Err(ValidationError::ZeroNodeId); }
    if !seen.insert(node.id) { return Err(ValidationError::DuplicateId(node.id)); }
    if node.text.len() > MAX_TEXT_LEN
        || node.props.placeholder.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.props.value.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.accessibility.name.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.accessibility.description.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.accessibility.role_description.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.accessibility.states.value_text.as_ref().is_some_and(|s| s.len() > MAX_TEXT_LEN)
        || node.accessibility.key_shortcuts.iter().any(|s| s.len() > 256) {
        return Err(ValidationError::TextTooLong);
    }
    if let Some(uri) = &node.props.resource {
        if !(uri.starts_with("app://") || uri.starts_with("https://")) {
            return Err(ValidationError::InvalidResourceScheme);
        }
    }
    validate_layout(&node.layout)?;
    validate_style(&node.style)?;
    validate_accessibility(&node.accessibility, node.kind)?;
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

fn implicit_accessibility_role(kind: NodeKind) -> AccessibilityRole {
    match kind {
        NodeKind::Button => AccessibilityRole::Button,
        NodeKind::Input => AccessibilityRole::Textbox,
        NodeKind::Image => AccessibilityRole::Image,
        NodeKind::List => AccessibilityRole::List,
        _ => AccessibilityRole::Generic,
    }
}

fn role_prohibits_name(role: AccessibilityRole) -> bool {
    matches!(
        role,
        AccessibilityRole::Caption
            | AccessibilityRole::Code
            | AccessibilityRole::Deletion
            | AccessibilityRole::Emphasis
            | AccessibilityRole::Generic
            | AccessibilityRole::Insertion
            | AccessibilityRole::None
            | AccessibilityRole::Paragraph
            | AccessibilityRole::Presentation
            | AccessibilityRole::Strong
            | AccessibilityRole::Subscript
            | AccessibilityRole::Superscript
    )
}

fn role_supports_invalid(role: AccessibilityRole) -> bool {
    matches!(
        role,
        AccessibilityRole::Application
            | AccessibilityRole::Checkbox
            | AccessibilityRole::Combobox
            | AccessibilityRole::GridCell
            | AccessibilityRole::ListBox
            | AccessibilityRole::RadioGroup
            | AccessibilityRole::Slider
            | AccessibilityRole::SpinButton
            | AccessibilityRole::Textbox
            | AccessibilityRole::Tree
            | AccessibilityRole::ColumnHeader
            | AccessibilityRole::RowHeader
            | AccessibilityRole::Searchbox
            | AccessibilityRole::Switch
            | AccessibilityRole::TreeGrid
    )
}

fn validate_accessibility(
    accessibility: &Accessibility,
    node_kind: NodeKind,
) -> Result<(), ValidationError> {
    let role = accessibility.role.unwrap_or_else(|| implicit_accessibility_role(node_kind));
    let states = &accessibility.states;

    if accessibility.name.as_ref().is_some_and(|name| name.trim().is_empty()) {
        return Err(ValidationError::EmptyAccessibilityName);
    }
    if accessibility.name.is_some() && role_prohibits_name(role) {
        return Err(ValidationError::NameProhibitedForRole);
    }
    if accessibility
        .role_description
        .as_ref()
        .is_some_and(|description| description.trim().is_empty())
    {
        return Err(ValidationError::EmptyAccessibilityRoleDescription);
    }

    if matches!(
        role,
        AccessibilityRole::Checkbox
            | AccessibilityRole::Radio
            | AccessibilityRole::Switch
            | AccessibilityRole::MenuItemCheckbox
            | AccessibilityRole::MenuItemRadio
    ) && states.checked.is_none()
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.checked.is_some()
        && !matches!(
            role,
            AccessibilityRole::Checkbox
                | AccessibilityRole::Radio
                | AccessibilityRole::Switch
                | AccessibilityRole::MenuItemCheckbox
                | AccessibilityRole::MenuItemRadio
                | AccessibilityRole::Option
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.checked == Some(CheckedState::Mixed)
        && !matches!(
            role,
            AccessibilityRole::Checkbox | AccessibilityRole::MenuItemCheckbox
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.pressed.is_some() && role != AccessibilityRole::Button {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.selected.is_some()
        && !matches!(
            role,
            AccessibilityRole::GridCell
                | AccessibilityRole::Option
                | AccessibilityRole::Row
                | AccessibilityRole::Tab
                | AccessibilityRole::TreeItem
                | AccessibilityRole::ColumnHeader
                | AccessibilityRole::RowHeader
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.expanded.is_some()
        && !matches!(
            role,
            AccessibilityRole::Application
                | AccessibilityRole::Button
                | AccessibilityRole::Checkbox
                | AccessibilityRole::Combobox
                | AccessibilityRole::GridCell
                | AccessibilityRole::Link
                | AccessibilityRole::ListBox
                | AccessibilityRole::MenuItem
                | AccessibilityRole::Row
                | AccessibilityRole::RowHeader
                | AccessibilityRole::Tab
                | AccessibilityRole::TreeItem
                | AccessibilityRole::ColumnHeader
                | AccessibilityRole::MenuItemCheckbox
                | AccessibilityRole::MenuItemRadio
                | AccessibilityRole::Switch
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.has_popup.is_some()
        && !matches!(
            role,
            AccessibilityRole::Application
                | AccessibilityRole::Button
                | AccessibilityRole::Combobox
                | AccessibilityRole::GridCell
                | AccessibilityRole::Link
                | AccessibilityRole::MenuItem
                | AccessibilityRole::Slider
                | AccessibilityRole::Tab
                | AccessibilityRole::Textbox
                | AccessibilityRole::TreeItem
                | AccessibilityRole::ColumnHeader
                | AccessibilityRole::MenuItemCheckbox
                | AccessibilityRole::MenuItemRadio
                | AccessibilityRole::RowHeader
                | AccessibilityRole::Searchbox
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.required
        && !matches!(
            role,
            AccessibilityRole::Checkbox
                | AccessibilityRole::Combobox
                | AccessibilityRole::GridCell
                | AccessibilityRole::ListBox
                | AccessibilityRole::RadioGroup
                | AccessibilityRole::SpinButton
                | AccessibilityRole::Textbox
                | AccessibilityRole::Tree
                | AccessibilityRole::ColumnHeader
                | AccessibilityRole::RowHeader
                | AccessibilityRole::Searchbox
                | AccessibilityRole::Switch
                | AccessibilityRole::TreeGrid
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.read_only
        && !matches!(
            role,
            AccessibilityRole::Checkbox
                | AccessibilityRole::Combobox
                | AccessibilityRole::Grid
                | AccessibilityRole::GridCell
                | AccessibilityRole::ListBox
                | AccessibilityRole::RadioGroup
                | AccessibilityRole::Slider
                | AccessibilityRole::SpinButton
                | AccessibilityRole::Textbox
                | AccessibilityRole::ColumnHeader
                | AccessibilityRole::RowHeader
                | AccessibilityRole::Searchbox
                | AccessibilityRole::Switch
                | AccessibilityRole::TreeGrid
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.invalid.is_some() && !role_supports_invalid(role) {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.modal
        && !matches!(role, AccessibilityRole::Dialog | AccessibilityRole::AlertDialog)
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.multiline.is_some()
        && !matches!(role, AccessibilityRole::Textbox | AccessibilityRole::Searchbox)
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.multiselectable.is_some()
        && !matches!(
            role,
            AccessibilityRole::Grid
                | AccessibilityRole::ListBox
                | AccessibilityRole::Tree
                | AccessibilityRole::TreeGrid
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.orientation.is_some()
        && !matches!(
            role,
            AccessibilityRole::Menu
                | AccessibilityRole::MenuBar
                | AccessibilityRole::ListBox
                | AccessibilityRole::RadioGroup
                | AccessibilityRole::Scrollbar
                | AccessibilityRole::Slider
                | AccessibilityRole::Separator
                | AccessibilityRole::TabList
                | AccessibilityRole::Toolbar
                | AccessibilityRole::Tree
                | AccessibilityRole::TreeGrid
                | AccessibilityRole::Grid
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.level.is_some()
        && !matches!(
            role,
            AccessibilityRole::Heading
                | AccessibilityRole::ListItem
                | AccessibilityRole::Row
                | AccessibilityRole::TreeItem
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if (states.position_in_set.is_some() || states.set_size.is_some())
        && !matches!(
            role,
            AccessibilityRole::Article
                | AccessibilityRole::ListItem
                | AccessibilityRole::MenuItem
                | AccessibilityRole::Option
                | AccessibilityRole::Radio
                | AccessibilityRole::Row
                | AccessibilityRole::Tab
                | AccessibilityRole::MenuItemCheckbox
                | AccessibilityRole::MenuItemRadio
                | AccessibilityRole::TreeItem
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.sort.is_some()
        && !matches!(role, AccessibilityRole::ColumnHeader | AccessibilityRole::RowHeader)
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if states.autocomplete.is_some()
        && !matches!(
            role,
            AccessibilityRole::Combobox
                | AccessibilityRole::Textbox
                | AccessibilityRole::Searchbox
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }

    if states.level == Some(0)
        || states.position_in_set == Some(0)
        || states.set_size == Some(0)
        || states.set_size.is_some_and(|size| size < -1)
    {
        return Err(ValidationError::InvalidAccessibilityRange);
    }
    if let (Some(position), Some(size)) = (states.position_in_set, states.set_size) {
        if size > 0 && position > size as u32 {
            return Err(ValidationError::InvalidAccessibilityRange);
        }
    }

    let range_values = [states.value_min, states.value_max, states.value_now];
    if range_values.iter().flatten().any(|value| !value.is_finite()) {
        return Err(ValidationError::InvalidNumber);
    }
    let has_range_value = states.value_min.is_some()
        || states.value_max.is_some()
        || states.value_now.is_some()
        || states.value_text.is_some();
    if has_range_value
        && !matches!(
            role,
            AccessibilityRole::Slider
                | AccessibilityRole::SpinButton
                | AccessibilityRole::ProgressBar
                | AccessibilityRole::Meter
                | AccessibilityRole::Scrollbar
        )
    {
        return Err(ValidationError::InvalidAccessibilityState);
    }
    if let (Some(min), Some(max)) = (states.value_min, states.value_max) {
        if min > max {
            return Err(ValidationError::InvalidAccessibilityRange);
        }
    }
    if let Some(now) = states.value_now {
        if states.value_min.is_some_and(|min| now < min)
            || states.value_max.is_some_and(|max| now > max)
        {
            return Err(ValidationError::InvalidAccessibilityRange);
        }
    }

    if states.row_count.is_some_and(|count| count < -1)
        || states.column_count.is_some_and(|count| count < -1)
        || states.row_index == Some(0)
        || states.column_index == Some(0)
        || states.row_span == Some(0)
        || states.column_span == Some(0)
    {
        return Err(ValidationError::InvalidAccessibilityRange);
    }
    if let (Some(count), Some(index)) = (states.row_count, states.row_index) {
        if count >= 0 && index > count as u32 {
            return Err(ValidationError::InvalidAccessibilityRange);
        }
    }
    if let (Some(count), Some(index)) = (states.column_count, states.column_index) {
        if count >= 0 && index > count as u32 {
            return Err(ValidationError::InvalidAccessibilityRange);
        }
    }
    if let (Some(index), Some(span), Some(count)) =
        (states.row_index, states.row_span, states.row_count)
    {
        if count >= 0 && index.saturating_add(span - 1) > count as u32 {
            return Err(ValidationError::InvalidAccessibilityRange);
        }
    }
    if let (Some(index), Some(span), Some(count)) =
        (states.column_index, states.column_span, states.column_count)
    {
        if count >= 0 && index.saturating_add(span - 1) > count as u32 {
            return Err(ValidationError::InvalidAccessibilityRange);
        }
    }

    if let Some(live_region) = &accessibility.live_region {
        if live_region.relevant.contains(&LiveRelevant::All) && live_region.relevant.len() > 1 {
            return Err(ValidationError::InvalidLiveRegionConfiguration);
        }
    }

    Ok(())
}

fn validate_reference_list(
    references: &[u64],
    known_ids: &HashSet<u64>,
) -> Result<(), ValidationError> {
    for reference in references {
        if *reference == 0 {
            return Err(ValidationError::ZeroNodeId);
        }
        if !known_ids.contains(reference) {
            return Err(ValidationError::UnknownAccessibilityReference(*reference));
        }
    }
    Ok(())
}

fn collect_tree_edges(
    node: &Node,
    graph: &mut HashMap<u64, Vec<u64>>,
    parent_by_id: &mut HashMap<u64, u64>,
) {
    graph.entry(node.id).or_default();
    for child in &node.children {
        graph.entry(node.id).or_default().push(child.id);
        parent_by_id.insert(child.id, node.id);
        collect_tree_edges(child, graph, parent_by_id);
    }
}

fn collect_accessibility_relationships(
    node: &Node,
    known_ids: &HashSet<u64>,
    owners: &mut HashMap<u64, u64>,
    active_descendants: &mut HashMap<u64, u64>,
) -> Result<(), ValidationError> {
    let relationships = &node.accessibility.relationships;
    validate_reference_list(&relationships.labelled_by, known_ids)?;
    let effective_role = node
        .accessibility
        .role
        .unwrap_or_else(|| implicit_accessibility_role(node.kind));
    if !relationships.labelled_by.is_empty() && role_prohibits_name(effective_role) {
        return Err(ValidationError::NameProhibitedForRole);
    }
    validate_reference_list(&relationships.described_by, known_ids)?;
    validate_reference_list(&relationships.controls, known_ids)?;
    validate_reference_list(&relationships.owns, known_ids)?;
    validate_reference_list(&relationships.details, known_ids)?;
    validate_reference_list(&relationships.flow_to, known_ids)?;
    validate_reference_list(&relationships.headers, known_ids)?;

    for reference in [relationships.active_descendant, relationships.error_message]
        .into_iter()
        .flatten()
    {
        if reference == 0 {
            return Err(ValidationError::ZeroNodeId);
        }
        if !known_ids.contains(&reference) {
            return Err(ValidationError::UnknownAccessibilityReference(reference));
        }
    }

    if let Some(active_descendant) = relationships.active_descendant {
        active_descendants.insert(node.id, active_descendant);
    }

    for owned_id in &relationships.owns {
        if let Some(previous_owner) = owners.insert(*owned_id, node.id) {
            if previous_owner != node.id {
                return Err(ValidationError::MultipleAccessibilityOwners(*owned_id));
            }
        }
    }

    for child in &node.children {
        collect_accessibility_relationships(child, known_ids, owners, active_descendants)?;
    }
    Ok(())
}

fn is_accessibility_descendant(
    graph: &HashMap<u64, Vec<u64>>,
    ancestor_id: u64,
    target_id: u64,
) -> bool {
    let mut visited = HashSet::new();
    let mut pending = vec![ancestor_id];

    while let Some(node_id) = pending.pop() {
        if !visited.insert(node_id) {
            continue;
        }
        if let Some(children) = graph.get(&node_id) {
            for child_id in children {
                if *child_id == target_id {
                    return true;
                }
                pending.push(*child_id);
            }
        }
    }
    false
}

fn validate_accessibility_relationships(
    root: &Node,
    known_ids: &HashSet<u64>,
) -> Result<(), ValidationError> {
    let mut graph = HashMap::new();
    let mut parent_by_id = HashMap::new();
    let mut owners = HashMap::new();
    let mut active_descendants = HashMap::new();

    collect_tree_edges(root, &mut graph, &mut parent_by_id);
    collect_accessibility_relationships(
        root,
        known_ids,
        &mut owners,
        &mut active_descendants,
    )?;

    // aria-owns changes the accessibility parent. Remove a prior tree edge
    // when the owner is not already the node's tree parent.
    for (owned_id, owner_id) in owners {
        if let Some(previous_parent) = parent_by_id.get(&owned_id) {
            if *previous_parent != owner_id {
                if let Some(children) = graph.get_mut(previous_parent) {
                    children.retain(|child_id| *child_id != owned_id);
                }
            }
        }
        let children = graph.entry(owner_id).or_default();
        if !children.contains(&owned_id) {
            children.push(owned_id);
        }
    }

    if let Some(node_id) = find_ownership_cycle(&graph) {
        return Err(ValidationError::AccessibilityOwnershipCycle(node_id));
    }

    for (owner_id, active_descendant_id) in active_descendants {
        if !is_accessibility_descendant(&graph, owner_id, active_descendant_id) {
            return Err(ValidationError::InvalidActiveDescendant(active_descendant_id));
        }
    }

    Ok(())
}

fn find_ownership_cycle(graph: &HashMap<u64, Vec<u64>>) -> Option<u64> {
    let mut state: HashMap<u64, u8> = HashMap::new();

    for start in graph.keys().copied() {
        if state.get(&start) == Some(&2) {
            continue;
        }

        let mut stack = vec![(start, 0usize)];
        state.insert(start, 1);

        while let Some((node_id, child_index)) = stack.pop() {
            let children = graph.get(&node_id).map(Vec::as_slice).unwrap_or(&[]);
            if child_index >= children.len() {
                state.insert(node_id, 2);
                continue;
            }

            stack.push((node_id, child_index + 1));
            let child_id = children[child_index];
            match state.get(&child_id).copied().unwrap_or(0) {
                1 => return Some(child_id),
                2 => {}
                _ => {
                    state.insert(child_id, 1);
                    stack.push((child_id, 0));
                }
            }
        }
    }
    None
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
    #[test]
    fn accessibility_relationships_must_resolve_inside_the_tree() {
        let mut root = sample_tree();
        root.accessibility.relationships.labelled_by.push(99);
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root }),
            Err(ValidationError::UnknownAccessibilityReference(99))
        );
    }

    #[test]
    fn accessibility_reference_lists_allow_repetition_and_require_resolved_ids() {
        let mut repeated = sample_tree();
        repeated.accessibility.role = Some(AccessibilityRole::Button);
        repeated.accessibility.relationships.labelled_by = vec![2, 2];
        assert!(validate_snapshot(&Snapshot { version: 1, root: repeated }).is_ok());
    }

    #[test]
    fn accessibility_owns_reparents_descendants_and_detects_tree_cycles() {
        let mut root = sample_tree();
        root.children[1].accessibility.relationships.owns = vec![2];
        assert!(validate_snapshot(&Snapshot { version: 1, root: root.clone() }).is_ok());

        root.children[0].accessibility.relationships.owns = vec![3];
        assert!(matches!(
            validate_snapshot(&Snapshot { version: 1, root }),
            Err(ValidationError::AccessibilityOwnershipCycle(_))
        ));
    }

    #[test]
    fn active_descendant_must_be_owned_or_a_tree_descendant() {
        let mut root = sample_tree();
        root.accessibility.role = Some(AccessibilityRole::ListBox);
        root.children[0].accessibility.role = Some(AccessibilityRole::Option);
        root.accessibility.relationships.active_descendant = Some(2);
        assert!(validate_snapshot(&Snapshot { version: 1, root: root.clone() }).is_ok());

        root.accessibility.relationships.active_descendant = Some(1);
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root }),
            Err(ValidationError::InvalidActiveDescendant(1))
        );
    }

    #[test]
    fn global_accessibility_states_are_not_overrestricted_by_role() {
        let mut root = sample_tree();
        root.accessibility.states.current = Some(CurrentState::Page);
        root.accessibility.states.busy = true;
        assert!(validate_snapshot(&Snapshot { version: 1, root }).is_ok());
    }

    #[test]
    fn typed_role_catalog_and_name_prohibitions_are_explicit() {
        let mut decorative = Node::new(1, NodeKind::Box);
        decorative.accessibility.role = Some(AccessibilityRole::Presentation);
        decorative.accessibility.name = Some("Decorative".into());
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root: decorative }),
            Err(ValidationError::NameProhibitedForRole)
        );

        let mut math = Node::new(1, NodeKind::Text);
        math.accessibility.role = Some(AccessibilityRole::Math);
        assert!(validate_snapshot(&Snapshot { version: 1, root: math }).is_ok());
    }

    #[test]
    fn accessibility_owns_relationships_reject_multiple_owners_and_cycles() {
        let mut multiple_owners = sample_tree();
        multiple_owners.accessibility.relationships.owns = vec![2];
        multiple_owners.children[1].accessibility.relationships.owns = vec![2];
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root: multiple_owners }),
            Err(ValidationError::MultipleAccessibilityOwners(2))
        );

        let mut cycle = sample_tree();
        cycle.accessibility.relationships.owns = vec![2];
        cycle.children[0].accessibility.relationships.owns = vec![1];
        assert!(matches!(
            validate_snapshot(&Snapshot { version: 1, root: cycle }),
            Err(ValidationError::AccessibilityOwnershipCycle(_))
        ));
    }

    #[test]
    fn accessibility_states_must_match_the_effective_role() {
        let mut root = sample_tree();
        root.accessibility.role = Some(AccessibilityRole::Textbox);
        root.accessibility.states.checked = Some(CheckedState::True);
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root }),
            Err(ValidationError::InvalidAccessibilityState)
        );

        let mut button = Node::new(7, NodeKind::Button);
        button.accessibility.states.pressed = Some(PressedState::Mixed);
        assert!(validate_snapshot(&Snapshot { version: 1, root: button }).is_ok());

        let mut option = Node::new(8, NodeKind::Text);
        option.accessibility.role = Some(AccessibilityRole::Option);
        option.accessibility.states.checked = Some(CheckedState::True);
        assert!(validate_snapshot(&Snapshot { version: 1, root: option }).is_ok());

        let mut checkbox = Node::new(9, NodeKind::Input);
        checkbox.accessibility.role = Some(AccessibilityRole::Checkbox);
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root: checkbox }),
            Err(ValidationError::InvalidAccessibilityState)
        );
    }

    #[test]
    fn accessibility_range_values_are_finite_and_consistent() {
        let mut root = Node::new(1, NodeKind::Box);
        root.accessibility.role = Some(AccessibilityRole::Slider);
        root.accessibility.states.value_min = Some(0.0);
        root.accessibility.states.value_max = Some(10.0);
        root.accessibility.states.value_now = Some(11.0);
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root }),
            Err(ValidationError::InvalidAccessibilityRange)
        );
    }

    #[test]
    fn accessibility_names_are_nonempty_and_bounded() {
        let mut empty = Node::new(1, NodeKind::Button);
        empty.accessibility.name = Some("  ".into());
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root: empty }),
            Err(ValidationError::EmptyAccessibilityName)
        );

        let mut long = Node::new(1, NodeKind::Button);
        long.accessibility.name = Some("x".repeat(MAX_TEXT_LEN + 1));
        assert_eq!(
            validate_snapshot(&Snapshot { version: 1, root: long }),
            Err(ValidationError::TextTooLong)
        );
    }

}
