//! `furnicad-geom` — low level mathematics and planar geometry.
//!
//! Everything here is pure Rust and deterministic: points/vectors wrap `glam`
//! doubles, [`contour::Contour2D`] models closed panel profiles made of lines
//! and arcs, [`tessellate`] turns contours into triangle meshes for rendering,
//! [`offset`] performs polygon offsetting used by edge bands and grooves.

pub mod contour;
pub mod curve;
pub mod mesh;
pub mod offset;
pub mod tessellate;

pub use contour::{Contour2D, Segment2D};
pub use curve::{Arc2D, LineSeg2D};
pub use mesh::{Mesh3D, Vertex};
