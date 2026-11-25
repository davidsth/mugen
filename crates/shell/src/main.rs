//! Mugen browser — v0.1: a real window, not a placeholder.
//!
//! This is the browser-only repo. Apps live in SEPARATE repos, run as
//! their own web servers (see the mugen-demo-app repo), and are added
//! to this browser locally by IP:port — exactly like adding a local
//! web app you are developing.
//!
//! How local adding works in v0.1:
//!   cargo run -p mugen-shell -- --app 127.0.0.1:4000
//!   MUGEN_LOCAL_APP=127.0.0.1:4000 cargo run -p mugen-shell
//!
//! That registers the endpoint as a joined app: a new sidebar icon
//! appears, and the browser logs it. It does NOT yet fetch/render the
//! app's Snapshot over WebSocket — that is the immediate next step
//! (Step 5). Until then, a local app's pane is a placeholder layout,
//! and the demo server can be verified separately with curl (see the
//! demo repo README). Claiming otherwise would be dishonest.
//!
//! Rendering here is softbuffer (CPU pixels) on purpose for v0.1:
//! no GPU shaders, no HTML/CSS/JS, just a pixel buffer you own.
//! The wgpu renderer in crates/renderer replaces this in Step 3,
//! keeping the same layout/draw ideas.

use std::{env, num::NonZeroU32, rc::Rc};

use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowId},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Tasks,
    Store,
    Atlas,
    Pulse,
    Local,
}

#[derive(Clone)]
struct JoinedApp {
    name: &'static str,
    colour: u32,
    kind: Kind,
    /// Local web-server endpoint, e.g. "127.0.0.1:4000".
    /// None for the built-in demo panes.
    endpoint: Option<String>,
}

fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r << 16) | (g << 8) | b
}

struct Frame<'a> {
    pixels: &'a mut [u32],
    width: usize,
    height: usize,
}

impl Frame<'_> {
    fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, colour: u32) {
        let x_end = x.saturating_add(width).min(self.width);
        let y_end = y.saturating_add(height).min(self.height);
        for py in y..y_end {
            for px in x..x_end {
                self.pixels[py * self.width + px] = colour;
            }
        }
    }
}

/// Parse local apps from `--app <host:port>`, `--app=<host:port>`,
/// and the MUGEN_LOCAL_APP env var. Accepts "127.0.0.1:4000",
/// "localhost:4000", or a full "http://127.0.0.1:4000" (scheme stripped).
fn local_apps_from_args() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Ok(v) = env::var("MUGEN_LOCAL_APP") {
        if !v.trim().is_empty() {
            out.push(v.trim().to_string());
        }
    }
    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--app" {
            if let Some(v) = args.get(i + 1) {
                out.push(v.clone());
                i += 2;
                continue;
            }
        } else if let Some(v) = args[i].strip_prefix("--app=") {
            out.push(v.to_string());
        }
        i += 1;
    }
    out.into_iter()
        .map(|v| {
            v.trim()
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .trim_end_matches('/')
                .to_string()
        })
        .filter(|v| !v.is_empty())
        .collect()
}

fn built_in_apps() -> Vec<JoinedApp> {
    vec![
        JoinedApp {
            name: "Tasks",
            colour: 0x2563eb,
            kind: Kind::Tasks,
            endpoint: None,
        },
        JoinedApp {
            name: "Store",
            colour: 0x059669,
            kind: Kind::Store,
            endpoint: None,
        },
        JoinedApp {
            name: "Atlas",
            colour: 0x7c3aed,
            kind: Kind::Atlas,
            endpoint: None,
        },
        JoinedApp {
            name: "Pulse",
            colour: 0xe11d48,
            kind: Kind::Pulse,
            endpoint: None,
        },
    ]
}

struct App {
    context: Context<OwnedDisplayHandle>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    apps: Vec<JoinedApp>,
    selected: usize,
    mouse_x: f64,
    mouse_y: f64,
}

impl App {
    fn draw(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(surface) = &mut self.surface else { return };
        let Some(active) = self.apps.get(self.selected).cloned() else {
            return;
        };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        surface
            .resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            )
            .unwrap();

        let mut buffer = surface.buffer_mut().unwrap();
        let w = buffer.width().get() as usize;
        let h = buffer.height().get() as usize;
        let mut frame = Frame { pixels: &mut *buffer, width: w, height: h };
        let scale = window.scale_factor();
        let s = |v: f32| (v * scale as f32) as usize;

        let bg = rgb(0xf3, 0xf4, 0xf6);
        let sidebar_w = s(72.0);
        for px in buffer.iter_mut() {
            *px = bg;
        }
        frame.fill_rect(0, 0, sidebar_w, h, rgb(0x11, 0x18, 0x27));

        let icon = s(48.0);
        let gap = s(12.0);
        let pad = s(12.0);
        for (i, app) in self.apps.iter().enumerate() {
            let y = pad + i * (icon + gap);
            if y + icon > h {
                break;
            }
            if i == self.selected {
                frame.fill_rect(pad - s(3.0),
                    y - s(3.0),
                    icon + s(6.0),
                    icon + s(6.0),
                    rgb(0xff, 0xff, 0xff),
                );
            }
            frame.fill_rect(pad, y, icon, icon, app.colour);
        }

        let main_x = sidebar_w;
        let main_w = w.saturating_sub(main_x);
        let header_h = s(56.0);
        frame.fill_rect(main_x,
            0,
            main_w,
            header_h,
            rgb(0xff, 0xff, 0xff),
        );
        frame.fill_rect(main_x,
            header_h,
            main_w,
            s(4.0),
            active.colour,
        );

        let content_y = header_h + s(24.0);
        let content_x = main_x + s(24.0);
        let content_w = main_w.saturating_sub(s(48.0));

        // Placeholder layouts per app kind. In Step 5 these rectangles
        // are replaced by the Snapshot tree sent by that app's server.
        match active.kind {
            Kind::Tasks => {
                for row in 0..5 {
                    let y = content_y + row * s(64.0);
                    frame.fill_rect(content_x,
                        y,
                        content_w,
                        s(48.0),
                        rgb(0xff, 0xff, 0xff),
                    );
                    frame.fill_rect(content_x + s(12.0),
                        y + s(12.0),
                        s(24.0),
                        s(24.0),
                        active.colour,
                    );
                    frame.fill_rect(content_x + s(48.0),
                        y + s(18.0),
                        content_w / 2,
                        s(12.0),
                        rgb(0xd1, 0xd5, 0xdb),
                    );
                }
            }
            Kind::Store => {
                let cols = 3;
                let card_w = content_w / cols - s(12.0);
                for card in 0..6 {
                    let col = card % cols;
                    let row = card / cols;
                    let x = content_x + col * (card_w + s(12.0));
                    let y = content_y + row * (s(180.0) + s(12.0));
                    frame.fill_rect(x,
                        y,
                        card_w,
                        s(180.0),
                        rgb(0xff, 0xff, 0xff),
                    );
                    frame.fill_rect(x,
                        y,
                        card_w,
                        s(110.0),
                        active.colour,
                    );
                }
            }
            Kind::Atlas => {
                for tab in 0..3 {
                    let x = content_x + tab * (content_w / 3);
                    frame.fill_rect(x,
                        content_y,
                        content_w / 3 - s(8.0),
                        s(36.0),
                        if tab == 1 {
                            active.colour
                        } else {
                            rgb(0xff, 0xff, 0xff)
                        },
                    );
                }
                for row in 0..4 {
                    let y = content_y + s(56.0) + row * s(64.0);
                    frame.fill_rect(content_x,
                        y,
                        content_w,
                        s(48.0),
                        rgb(0xff, 0xff, 0xff),
                    );
                }
            }
            Kind::Pulse => {
                for card in 0..3 {
                    let y = content_y + card * (s(120.0) + s(12.0));
                    frame.fill_rect(content_x,
                        y,
                        content_w,
                        s(120.0),
                        rgb(0xff, 0xff, 0xff),
                    );
                    frame.fill_rect(content_x + s(12.0),
                        y + s(12.0),
                        s(36.0),
                        s(36.0),
                        active.colour,
                    );
                }
            }
            Kind::Local => {
                // Local app pane (v0.1 placeholder): a large card + rows.
                // Step 5 replaces this with the server's Snapshot tree.
                frame.fill_rect(content_x,
                    content_y,
                    content_w,
                    s(120.0),
                    rgb(0xff, 0xff, 0xff),
                );
                frame.fill_rect(content_x,
                    content_y,
                    content_w,
                    s(8.0),
                    active.colour,
                );
                for row in 0..4 {
                    let y = content_y + s(144.0) + row * s(64.0);
                    frame.fill_rect(content_x,
                        y,
                        content_w,
                        s(48.0),
                        rgb(0xff, 0xff, 0xff),
                    );
                }
            }
        }

        drop(frame);
        buffer.present().unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Mugen")
                        .with_inner_size(LogicalSize::new(960, 640)),
                )
                .unwrap(),
        );
        let surface = Surface::new(&self.context, window.clone()).unwrap();
        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_x = position.x;
                self.mouse_y = position.y;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(window) = &self.window {
                    let scale = window.scale_factor();
                    let s = |v: f64| v * scale;
                    if self.mouse_x < s(72.0) {
                        let y = self.mouse_y - s(12.0);
                        let step = s(60.0);
                        if y >= 0.0 {
                            let idx = (y / step) as usize;
                            if idx < self.apps.len() {
                                self.selected = idx;
                                if let Some(app) = self.apps.get(idx) {
                                    match &app.endpoint {
                                        Some(ep) => println!(
                                            "Mugen: switched to local app {} at {}",
                                            app.name, ep
                                        ),
                                        None => println!(
                                            "Mugen: switched to {} (built-in demo pane)",
                                            app.name
                                        ),
                                    }
                                }
                                window.request_redraw();
                            }
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let mut apps = built_in_apps();
    for endpoint in local_apps_from_args() {
        println!("Mugen: added local app at {}", endpoint);
        println!("Mugen:   manifest: http://{}/manifest", endpoint);
        println!("Mugen:   websocket: ws://{}/ws", endpoint);
        println!("Mugen:   (v0.1 registers the endpoint; Snapshot rendering over WebSocket is the next step.)");
        apps.push(JoinedApp {
            name: "Local",
            colour: 0x0ea5e9,
            kind: Kind::Local,
            endpoint: Some(endpoint),
        });
    }

    println!("Mugen browser starting.");
    println!("Joined apps:");
    for app in &apps {
        match &app.endpoint {
            Some(ep) => println!("  - {} at {}", app.name, ep),
            None => println!("  - {} (built-in demo pane)", app.name),
        }
    }
    println!("Tip: cargo run -p mugen-shell -- --app 127.0.0.1:4000");

    let event_loop = EventLoop::new().unwrap();
    let context = Context::new(event_loop.owned_display_handle()).unwrap();
    let mut app = App {
        context,
        window: None,
        surface: None,
        apps,
        selected: 0,
        mouse_x: 0.0,
        mouse_y: 0.0,
    };
    event_loop.run_app(&mut app).unwrap();
}
