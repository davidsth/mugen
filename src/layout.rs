use taffy::TaffyTree;
use taffy::NodeId;
use taffy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LayoutNode {
    pub width: f32,
    pub height: f32,
    pub background: [u8; 3],
    #[serde(default)]
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn to_taffy(&self, taffy: &mut TaffyTree) -> (NodeId, Vec<(NodeId, [u8; 3])>) {
        let mut colored_nodes = Vec::new();

        let style = Style {
            size: Size {
                width: Dimension::Length(self.width),
                height: Dimension::Length(self.height),
            },
            ..Default::default()
        };

        let children: Vec<(NodeId, Vec<(NodeId, [u8; 3])>)> =
            self.children.iter().map(|c| c.to_taffy(taffy)).collect();

        let child_nodes: Vec<NodeId> = children.iter().map(|(node, _)| *node).collect();
        let node = taffy.new_with_children(style, &child_nodes).unwrap();

        colored_nodes.push((node, self.background));
        for (_, sub_nodes) in children {
            colored_nodes.extend(sub_nodes);
        }

        (node, colored_nodes)
    }
}