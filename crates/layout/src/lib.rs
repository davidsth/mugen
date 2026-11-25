//! Deterministic two-pass layout for Mugen protocol trees.
//!
//! This first implementation deliberately has no renderer dependency. It computes
//! geometry from typed protocol values and uses conservative intrinsic text sizing.
//! Text shaping/measurement can be injected in a later milestone.

use mugen_protocol::{Align, Axis, LayoutKind, Length, Node, NodeKind, Justify};

/// Computed geometry for a protocol node, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaidOut {
    pub node_id: u64,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Size { width: f32, height: f32 }

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect { x: f32, y: f32, width: f32, height: f32 }

/// Layout a tree within a logical-pixel viewport.
///
/// The root is constrained to the viewport, with its own explicit/min/max
/// sizing applied. Nodes are emitted in preorder. Invalid viewport values are
/// treated as zero to keep layout deterministic and avoid propagating NaNs.
pub fn layout(root: &Node, width: f32, height: f32) -> Vec<LaidOut> {
    let viewport = Size { width: sane(width), height: sane(height) };
    let root_size = resolve_size(root, viewport, measure(root, viewport));
    let root_rect = Rect { x: 0.0, y: 0.0, width: root_size.width.min(viewport.width), height: root_size.height.min(viewport.height) };
    let mut out = Vec::new();
    place(root, root_rect, &mut out);
    out
}

fn sane(v: f32) -> f32 { if v.is_finite() && v >= 0.0 { v } else { 0.0 } }
fn clamp(v: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let lo = min.map(sane).unwrap_or(0.0);
    let hi = max.map(sane).unwrap_or(f32::MAX).max(lo);
    sane(v).max(lo).min(hi)
}
fn resolve_length(length: Length, available: f32, intrinsic: f32) -> f32 {
    match length {
        Length::Auto => intrinsic,
        Length::Px(v) => sane(v),
        Length::Percent(v) => available * sane(v) / 100.0,
        Length::Fill => available,
    }
}
fn inner(parent: Size, node: &Node) -> Size {
    Size {
        width: (parent.width - node.layout.padding.left - node.layout.padding.right).max(0.0),
        height: (parent.height - node.layout.padding.top - node.layout.padding.bottom).max(0.0),
    }
}
fn measure(node: &Node, available: Size) -> Size {
    let content_available = inner(available, node);
    let gap = node.layout.gap;
    let child_sizes: Vec<Size> = node.children.iter().map(|child| measure(child, content_available)).collect();
    let content = match node.layout.kind {
        LayoutKind::Row => Size {
            width: child_sizes.iter().map(|s| s.width).sum::<f32>() + gap * child_sizes.len().saturating_sub(1) as f32,
            height: child_sizes.iter().map(|s| s.height).fold(0.0, f32::max),
        },
        LayoutKind::Column => Size {
            width: child_sizes.iter().map(|s| s.width).fold(0.0, f32::max),
            height: child_sizes.iter().map(|s| s.height).sum::<f32>() + gap * child_sizes.len().saturating_sub(1) as f32,
        },
        LayoutKind::Stack | LayoutKind::Grid => Size {
            width: child_sizes.iter().map(|s| s.width).fold(0.0, f32::max),
            height: child_sizes.iter().map(|s| s.height).fold(0.0, f32::max),
        },
    };
    let intrinsic = if node.kind == NodeKind::Text {
        // Approximate only: 8 logical px per character, 20 px line height.
        Size { width: node.text.chars().count() as f32 * 8.0, height: if node.text.is_empty() { 0.0 } else { 20.0 } }
    } else if node.children.is_empty() {
        Size { width: 0.0, height: 0.0 }
    } else { content };
    Size {
        width: intrinsic.width + node.layout.padding.left + node.layout.padding.right,
        height: intrinsic.height + node.layout.padding.top + node.layout.padding.bottom,
    }
}
fn resolve_size(node: &Node, available: Size, intrinsic: Size) -> Size {
    Size {
        width: clamp(resolve_length(node.layout.width, available.width, intrinsic.width), node.layout.min_width, node.layout.max_width),
        height: clamp(resolve_length(node.layout.height, available.height, intrinsic.height), node.layout.min_height, node.layout.max_height),
    }
}
fn place(node: &Node, rect: Rect, out: &mut Vec<LaidOut>) {
    out.push(LaidOut { node_id: node.id, x: rect.x, y: rect.y, width: rect.width, height: rect.height });
    if node.children.is_empty() { return; }
    let pad = node.layout.padding;
    let content = Rect {
        x: rect.x + pad.left, y: rect.y + pad.top,
        width: (rect.width - pad.left - pad.right).max(0.0),
        height: (rect.height - pad.top - pad.bottom).max(0.0),
    };
    let gap = node.layout.gap;
    let measured: Vec<Size> = node.children.iter().map(|c| measure(c, Size { width: content.width, height: content.height })).collect();
    if node.layout.kind == LayoutKind::Stack {
        for child in &node.children {
            let size = resolve_size(child, Size { width: content.width, height: content.height }, measure(child, Size { width: content.width, height: content.height }));
            let width = if child.layout.width == Length::Fill || node.layout.align == Align::Stretch { content.width } else { size.width.min(content.width) };
            let height = if child.layout.height == Length::Fill || node.layout.align == Align::Stretch { content.height } else { size.height.min(content.height) };
            let x = match node.layout.align { Align::Center => content.x + (content.width-width)/2.0, Align::End => content.x+content.width-width, _ => content.x };
            let y = match node.layout.align { Align::Center => content.y + (content.height-height)/2.0, Align::End => content.y+content.height-height, _ => content.y };
            place(child, Rect { x, y, width, height }, out);
        }
        return;
    }
    let main_axis = match node.layout.kind { LayoutKind::Row => Axis::Horizontal, LayoutKind::Column => Axis::Vertical, LayoutKind::Grid => Axis::Horizontal, _ => node.layout.axis };
    let main_available = if main_axis == Axis::Horizontal { content.width } else { content.height };
    let base_sum = measured.iter().map(|s| if main_axis == Axis::Horizontal { s.width } else { s.height }).sum::<f32>() + gap * node.children.len().saturating_sub(1) as f32;
    let free = (main_available - base_sum).max(0.0);
    let grow_sum = node.children.iter().map(|c| c.layout.grow).sum::<f32>();
    let justify_offset = match node.layout.justify {
        Justify::Start => 0.0,
        Justify::Center => (main_available - base_sum).max(0.0) / 2.0,
        Justify::End => (main_available - base_sum).max(0.0),
        _ => 0.0,
    };
    let extra_gap = match node.layout.justify {
        Justify::SpaceBetween if node.children.len() > 1 => free / (node.children.len() - 1) as f32,
        Justify::SpaceAround if !node.children.is_empty() => free / node.children.len() as f32,
        _ => 0.0,
    };
    let mut cursor = if node.layout.justify == Justify::SpaceAround { extra_gap / 2.0 } else { justify_offset };
    for (i, child) in node.children.iter().enumerate() {
        let child_intrinsic = measured[i];
        let child_available = Size { width: content.width, height: content.height };
        let child_size = resolve_size(child, child_available, child_intrinsic);
        let grow_extra = if grow_sum > 0.0 { free * child.layout.grow / grow_sum } else { 0.0 };
        let main_size = (if main_axis == Axis::Horizontal { child_size.width } else { child_size.height }) + grow_extra;
        let cross_available = if main_axis == Axis::Horizontal { content.height } else { content.width };
        let cross_intrinsic = if main_axis == Axis::Horizontal { child_size.height } else { child_size.width };
        let cross_size = if node.layout.align == Align::Stretch {
            cross_available
        } else { cross_intrinsic.min(cross_available) };
        let (x, y, width, height) = if main_axis == Axis::Horizontal {
            let y = match node.layout.align { Align::Center => content.y + (content.height-cross_size)/2.0, Align::End => content.y+content.height-cross_size, _ => content.y };
            (content.x + cursor, y, main_size.min(content.width), cross_size)
        } else {
            let x = match node.layout.align { Align::Center => content.x + (content.width-cross_size)/2.0, Align::End => content.x+content.width-cross_size, _ => content.x };
            (x, content.y + cursor, cross_size, main_size.min(content.height))
        };
        place(child, Rect { x, y, width, height }, out);
        cursor += main_size + gap + extra_gap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mugen_protocol::{Layout, Insets};

    fn node(id: u64, kind: LayoutKind, children: Vec<Node>) -> Node {
        let mut n = Node::new(id, NodeKind::Box);
        n.layout = Layout { kind, axis: Axis::Vertical, align: Align::Start, justify: Justify::Start,
            gap: 0.0, padding: Insets::ZERO, width: Length::Auto, height: Length::Auto,
            min_width: None, min_height: None, max_width: None, max_height: None, grow: 0.0, shrink: 1.0 };
        n.children = children;
        n
    }
    fn get(items: &[LaidOut], id: u64) -> LaidOut { *items.iter().find(|x| x.node_id == id).unwrap() }

    #[test] fn row_places_children_sequentially() {
        let root = node(1, LayoutKind::Row, vec![Node::text(2,"ab"), Node::text(3,"abc")]);
        let result = layout(&root, 100.0, 40.0);
        assert_eq!(get(&result,2).x, 0.0);
        assert_eq!(get(&result,2).width, 16.0);
        assert_eq!(get(&result,3).x, 16.0);
    }
    #[test] fn column_respects_gap_and_padding() {
        let mut root = node(1, LayoutKind::Column, vec![Node::text(2,"a"),Node::text(3,"b")]);
        root.layout.gap=5.0;
        root.layout.padding=Insets{top:3.0,right:0.0,bottom:0.0,left:4.0};
        let result=layout(&root,100.0,100.0);
        assert_eq!(get(&result,2).x,4.0);
        assert_eq!(get(&result,2).y,3.0);
        assert_eq!(get(&result,3).y,28.0);
    }
    #[test] fn stack_overlays_children() {
        let root=node(1,LayoutKind::Stack,vec![Node::text(2,"a"),Node::text(3,"b")]);
        let result=layout(&root,100.0,40.0);
        assert_eq!(get(&result,2).x,get(&result,3).x);
        assert_eq!(get(&result,2).y,get(&result,3).y);
    }
    #[test] fn invalid_viewport_is_sanitized() {
        let root=Node::new(1,NodeKind::Box);
        let result=layout(&root,f32::NAN,-10.0);
        assert_eq!(result[0].width,0.0);
        assert_eq!(result[0].height,0.0);
    }
}

