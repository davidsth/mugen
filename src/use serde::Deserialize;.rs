use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use taffy::prelude::*;
use pixels::{Pixels, SurfaceTexture};
use winit::event_loop::EventLoop;
use winit::{event::*, event_loop::ControlFlow, window::WindowBuilder};

#[derive(Debug, Deserialize)]
pub struct LayoutNode {
    pub r#type: String,
    pub id: Option<String>,
    pub props: HashMap<String, serde_json::Value>,
    pub layout: HashMap<String, serde_json::Value>,
    pub children: Vec<LayoutNode>,
}

fn parse_style(layout: &HashMap<String, serde_json::Value>) -> Style {
    Style {
        size: Size {
            width: layout
                .get("width")
                .and_then(|v| v.as_f64())
                .map(|w| Dimension::Length(w as f32))
                .unwrap_or(Dimension::Auto),
            height: layout
                .get("height")
                .and_then(|v| v.as_f64())
                .map(|h| Dimension::Length(h as f32))
                .unwrap_or(Dimension::Auto),
        },
        flex_direction: match layout.get("direction").and_then(|v| v.as_str()) {
            Some("row") => FlexDirection::Row,
            _ => FlexDirection::Column,
        },
        justify_content: match layout.get("justify_content").and_then(|v| v.as_str()) {
            Some("center") => Some(JustifyContent::Center),
            Some("end") => Some(JustifyContent::FlexEnd),
            _ => Some(JustifyContent::FlexStart),
        },
        align_items: match layout.get("align_items").and_then(|v| v.as_str()) {
            Some("center") => Some(AlignItems::Center),
            Some("end") => Some(AlignItems::FlexEnd),
            _ => Some(AlignItems::FlexStart),
        },
        ..Default::default()
    }
}

fn build_taffy_tree(taffy: &mut TaffyTree, node: &LayoutNode) -> Result<NodeId, taffy::TaffyError> {
    let style = parse_style(&node.layout);
    let children: Result<Vec<NodeId>, _> = node
        .children
        .iter()
        .map(|child| build_taffy_tree(taffy, child))
        .collect();

    taffy.new_with_children(style, &children?)
}


fn draw_layout(
    taffy: &mut TaffyTree,
    node_id: NodeId,
    frame: &mut [u8],
    width: u32,
    height: u32,
) {
    let layout = taffy.layout(node_id).unwrap();
    let x = layout.location.x as u32;
    let y = layout.location.y as u32;
    let w = layout.size.width as u32;
    let h = layout.size.height as u32;

    for row in y..y + h {
        for col in x..x + w {
            let idx = ((row * width + col) * 4) as usize;
            if idx + 3 < frame.len() {
                frame[idx] = 0xff;        // R
                frame[idx + 1] = 0x80;    // G
                frame[idx + 2] = 0x00;    // B
                frame[idx + 3] = 0xff;    // A
            }
        }
    }

    for child in taffy.children(node_id).unwrap() {
        draw_layout(taffy, child, frame, width, height);
    }
}

fn build_layout_tree(taffy: &mut TaffyTree) -> NodeId {
    let child = taffy
        .new_leaf(Style {
            size: Size {
                width: Dimension::Length(100.0),
                height: Dimension::Length(100.0),
            },
            ..Default::default()
        })
        .unwrap();

    taffy
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: Dimension::Length(300.0),
                    height: Dimension::Length(300.0),
                },
                justify_content: Some(JustifyContent::Center),
                align_items: Some(AlignItems::Center),
                ..Default::default()
            },
            &[child],
        )
        .unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Crystal UI Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(window_size.width, window_size.height, surface_texture).unwrap();

    let json = std::fs::read_to_string("crystal_app/layout.json").unwrap();
    let root_node = serde_json::from_str::<LayoutNode>(&json).unwrap();

    let mut taffy = TaffyTree::new();
    let root_id = build_layout_tree(&mut taffy);
    taffy.compute_layout(root_id, Size::MAX_CONTENT).unwrap();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();
                frame.fill(0); // Clear the screen to black

                draw_layout(
                    &mut taffy,
                    root_id,
                    frame,
                    window_size.width,
                    window_size.height,
                );

                pixels.render().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });

    Ok(())
}
