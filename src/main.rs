mod layout;

use taffy::TaffyTree;
use layout::LayoutNode;
use pixels::{Pixels, SurfaceTexture};
use std::{fs::File, io::BufReader};
use taffy::prelude::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rust GUI Viewer")
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();

    let file = File::open("src/layout.json").expect("Failed to open layout.json");
    let reader = BufReader::new(file);
    let root: LayoutNode = serde_json::from_reader(reader).unwrap();

    let mut taffy = TaffyTree::new();
    let (root_node, color_map) = root.to_taffy(&mut taffy);
    taffy.compute_layout(root_node, Size::MAX_CONTENT).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                let frame = pixels.frame_mut();
                for pixel in frame.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[30, 30, 60, 255]);
                }

                for (node, color) in &color_map {
                    if let Ok(layout) = taffy.layout(*node) {
                        let x = layout.location.x as u32;
                        let y = layout.location.y as u32;
                        let w = layout.size.width as u32;
                        let h = layout.size.height as u32;

                        for j in y..(y + h) {
                            for i in x..(x + w) {
                                if (i < size.width) && (j < size.height) {
                                    let idx = ((j * size.width + i) * 4) as usize;
                                    pixels.frame_mut()[idx..idx + 4].copy_from_slice(&[
                                        color[0], color[1], color[2], 255,
                                    ]);
                                }
                            }
                        }
                    }
                }

                pixels.render().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                pixels.resize_surface(size.width, size.height).unwrap();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}