//! `furnicad-kernel` — geometric kernel trait and the default pure-Rust
//! mesh-based implementation.
//!
//! The [`GeometryKernel`] trait mirrors what an Open CASCADE adapter
//! (`furnicad-kernel-occt`, optional FFI crate) would implement: extrude,
//! revolve, booleans, fillet/chamfer. The bundled [`MeshKernel`] performs all
//! operations on triangle meshes — adequate for furniture panels where exact
//! B-Rep is only needed for STEP I/O.

use furnicad_geom::{Contour2D, Mesh3D};
use serde::{Deserialize, Serialize};

/// A named extrudable/revolvable profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub contour: Contour2D,
}

impl Profile {
    pub fn from_contour(c: Contour2D) -> Self {
        Self { contour: c }
    }

    /// Rectangular panel profile.
    pub fn rect(w: f64, h: f64) -> Self {
        Self {
            contour: Contour2D::rect(w, h),
        }
    }
}

/// Rotation axis in local space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Axis {
    pub origin: [f64; 3],
    pub dir: [f64; 3],
}

impl Axis {
    pub const X: Axis = Axis {
        origin: [0.0; 3],
        dir: [1.0, 0.0, 0.0],
    };
    pub const Y: Axis = Axis {
        origin: [0.0; 3],
        dir: [0.0, 1.0, 0.0],
    };
    pub const Z: Axis = Axis {
        origin: [0.0; 3],
        dir: [0.0, 0.0, 1.0],
    };
}

/// Reference to an edge of a shape (endpoint pair used to locate it).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdgeRef {
    pub a: [f64; 3],
    pub b: [f64; 3],
}

/// Boolean operation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanKind {
    Union,
    Subtract,
    Intersect,
}

/// Kernel output shape: display mesh + lightweight construction notes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub mesh: Mesh3D,
    pub notes: Vec<String>,
}

/// The kernel contract every backend (mesh, OCCT FFI) must satisfy.
pub trait GeometryKernel {
    fn extrude(
        &self,
        profile: &Profile,
        direction: [f64; 3],
        depth: f64,
    ) -> furnicad_core::Result<Shape>;
    fn revolve(
        &self,
        profile: &Profile,
        axis: Axis,
        angle: f64,
    ) -> furnicad_core::Result<Shape>;
    fn boolean_union(&self, a: &Shape, b: &Shape) -> furnicad_core::Result<Shape>;
    fn boolean_subtract(&self, a: &Shape, b: &Shape) -> furnicad_core::Result<Shape>;
    fn boolean_intersect(&self, a: &Shape, b: &Shape) -> furnicad_core::Result<Shape>;
    fn fillet(
        &self,
        shape: &Shape,
        edges: &[EdgeRef],
        radius: f64,
    ) -> furnicad_core::Result<Shape>;
    fn chamfer(
        &self,
        shape: &Shape,
        edges: &[EdgeRef],
        size: f64,
    ) -> furnicad_core::Result<Shape>;
}

mod mesh_kernel;
pub use mesh_kernel::MeshKernel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_extrudes_box() {
        let k = MeshKernel::new();
        let s = k
            .extrude(&Profile::rect(10.0, 10.0), [0.0, 0.0, 1.0], 5.0)
            .unwrap();
        assert!((s.mesh.volume().abs() - 500.0).abs() < 1e-6);
    }
}
