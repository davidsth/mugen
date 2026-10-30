//! Step 3 placeholder.
//!
//! Implement here, on top of wgpu (NOT a hand-written GPU backend):
//! - Flatten a laid-out tree into a display list, sorted by layer.
//! - Cull off-screen nodes; batch by material.
//! - Texture atlas for glyphs and small images.
//! - Text via cosmic-text / HarfBuzz underneath — never write a shaper.
//! - Accessibility tree via accesskit — otherwise screen readers see
//!   a blank window, which is a real problem with custom renderers.
//!
//! Animate transforms / opacity in the display list, not by sending
//! a new tree 60 times a second from the Crystal server.
