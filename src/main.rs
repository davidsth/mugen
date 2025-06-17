use std::ops::Add;

use pixels::{Pixels, SurfaceTexture};
use taffy::prelude::*;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit::window::Window;

#[derive(Debug)]
enum WidgetType {
    Button,
    TextBox,
}

struct Widget {
    widget_type: WidgetType,
    node: NodeId,
    label: String,
    focused: bool,
}

impl Widget {
    fn contains(&self, layout: &Layout, x: f32, y: f32) -> bool {
        x >= layout.location.x
            && y >= layout.location.y
            && x <= layout.location.x + layout.size.width
            && y <= layout.location.y + layout.size.height
    }

    fn on_click(&mut self) {
        match self.widget_type {
            WidgetType::Button => println!("Button clicked!"),
            WidgetType::TextBox => {
                println!("TextBox focused!");
                self.focused = true;
            }
        }
    }

    fn color(&self) -> [u8; 4] {
        match self.widget_type {
            WidgetType::Button => [0xCC, 0x44, 0x44, 0xFF], // red
            WidgetType::TextBox => {
                if self.focused {
                    [0x44, 0xCC, 0x44, 0xFF] // green
                } else {
                    [0x44, 0x44, 0xCC, 0xFF] // blue
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Button + TextBox Demo")
        .with_inner_size(LogicalSize::new(400.0, 400.0))
        .build(&event_loop).unwrap();

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(window_size.width, window_size.height, surface_texture)?;

    let mut taffy = TaffyTree::new();
    let mut widgets = Vec::new();

    // Button
    let button_node = taffy.new_leaf(Style {
        size: Size {
            width: Dimension::Length(300.0),
            height: Dimension::Length(60.0),
        },
        margin: Rect {
            left: LengthPercentageAuto::Length(10.0),
            right: LengthPercentageAuto::Length(10.0),
            top: LengthPercentageAuto::Length(10.0),
            bottom: LengthPercentageAuto::Length(10.0),
        }
        .into(),
        ..Default::default()
    })?;
    widgets.push(Widget {
        widget_type: WidgetType::Button,
        node: button_node,
        label: "Click Me".into(),
        focused: false,
    });

    // TextBox
    let textbox_node = taffy.new_leaf(Style {
        size: Size {
            width: Dimension::Length(300.0),
            height: Dimension::Length(60.0),
        },
        margin: Rect {
            left: LengthPercentageAuto::Length(10.0),
            right: LengthPercentageAuto::Length(10.0),
            top: LengthPercentageAuto::Length(10.0),
            bottom: LengthPercentageAuto::Length(10.0),
        }
        .into(),
        ..Default::default()
    })?;
    widgets.push(Widget {
        widget_type: WidgetType::TextBox,
        node: textbox_node,
        label: "Type here...".into(),
        focused: false,
    });

    let root = taffy.new_with_children(
        Style {
            flex_direction: FlexDirection::Column,
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::Center),
            size: Size {
                width: Dimension::Length(window_size.width as f32),
                height: Dimension::Length(window_size.height as f32),
            },
            ..Default::default()
        },
        &widgets.iter().map(|w| w.node).collect::<Vec<_>>(),
    )?;

    taffy.compute_layout(root, Size::MAX_CONTENT)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                let frame = pixels.frame();
                for pixel in frame.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[0xDD, 0xDD, 0xDD, 0xFF]); // background
                }

                for widget in &widgets {
                    if let Ok(layout) = taffy.layout(widget.node) {
                        let x = layout.location.x as usize;
                        let y = layout.location.y as usize;
                        let w = layout.size.width as usize;
                        let h = layout.size.height as usize;
                        let color = widget.color();

                        for row in y..(y + h).min(window_size.height as usize) {
                            for col in x..(x + w).min(window_size.width as usize) {
                                let i = (row * window_size.width as usize + col) * 4;
                                if i + 3 < frame.len() {
                                    frame[i..i + 4].copy_from_slice(&color);
                                }
                            }
                        }
                    }
                }

                pixels.render().unwrap();
            }

            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } if state == ElementState::Pressed && button == MouseButton::Left => {
                if let Some(physical_pos) = 
                    .map(|p| p.to_physical(window.scale_factor()))
                {
                    let mx = physical_pos.x as f32;
                    let my = physical_pos.y as f32;

                    for widget in &mut widgets {
                        if let Ok(layout) = taffy.layout(widget.node) {
                            if widget.contains(layout, mx, my) {
                                widget.on_click();
                            } else if matches!(widget.widget_type, WidgetType::TextBox) {
                                widget.focused = false;
                            }
                        }
                    }

                    window.request_redraw();
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::MainEventsCleared => window.request_redraw(),
            _ => (),
        }
    });
}
