//! `furnicad-render` — viewport scene model.
//!
//! This crate is intentionally GPU-free: it builds the *scene description*
//! (meshes + display mode + camera + overlay lines) that a wgpu/egui shell in
//! `furnicad-app` draws. Keeping it pure makes unit testing and headless
//! snapshot checks possible (spec §18).

pub mod camera;
pub mod scene;

pub use camera::{Camera, Projection, ViewOrientation};
pub use scene::{DisplayMode, SceneItem, ViewportScene};
