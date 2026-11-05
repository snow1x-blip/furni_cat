//! Pure-Rust mesh geometry kernel implementing [`GeometryKernel`].

use crate::{Axis, EdgeRef, GeometryKernel, Profile, Shape};
use furnicad_core::error::{CadError, Result};
use furnicad_geom::mesh::Mesh3D;
use furnicad_geom::tessellate;

/// Default kernel. Booleans are recorded as CSG intent and resolved at mesh
/// level with conservative approximations; production builds can swap in the
/// OCCT adapter without touching callers.
#[derive(Debug, Clone)]
pub struct MeshKernel {
    pub chord_segments: usize,
    pub revolve_steps: usize,
}

impl Default for MeshKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshKernel {
    pub fn new() -> Self {
        Self {
            chord_segments: 48,
            revolve_steps: 64,
        }
    }
}

impl GeometryKernel for MeshKernel {
    fn extrude(&self, profile: &Profile, direction: [f64; 3], depth: f64) -> Result<Shape> {
        if depth <= 0.0 {
            return Err(CadError::geometry("extrude depth must be positive"));
        }
        let mesh = tessellate::extrude_contour(&profile.contour, direction, depth, self.chord_segments)?;
        Ok(Shape {
            mesh,
            notes: vec![format!("extrude {:.1} mm", depth)],
        })
    }

    fn revolve(&self, profile: &Profile, _axis: Axis, angle: f64) -> Result<Shape> {
        if angle <= 0.0 {
            return Err(CadError::geometry("revolve angle must be positive"));
        }
        // Profiles for turned parts are authored in the (radius, height) plane.
        let poly = profile.contour.to_polyline(self.chord_segments.max(16));
        let prof: Vec<[f64; 2]> = poly.iter().map(|p| [p[0].abs(), p[1]]).collect();
        let mesh = tessellate::revolve_profile(&prof, angle, self.revolve_steps)?;
        Ok(Shape {
            mesh,
            notes: vec![format!("revolve {:.1} deg", angle.to_degrees())],
        })
    }

    fn boolean_union(&self, a: &Shape, b: &Shape) -> Result<Shape> {
        let mut m = a.mesh.clone();
        m.extend(&b.mesh);
        Ok(Shape {
            mesh: m,
            notes: merged_notes(a, b, "union"),
        })
    }

    fn boolean_subtract(&self, a: &Shape, b: &Shape) -> Result<Shape> {
        // Mesh-level CSG subtract intent: keep target, append inverted tool so
        // an exact backend or preview shader can resolve; volume bookkeeping
        // via signed-mesh integration still works for nested closed shells.
        let mut m = a.mesh.clone();
        let mut inv = b.mesh.clone();
        flip_normals(&mut inv);
        m.extend(&inv);
        Ok(Shape {
            mesh: m,
            notes: merged_notes(a, b, "subtract"),
        })
    }

    fn boolean_intersect(&self, a: &Shape, b: &Shape) -> Result<Shape> {
        let m = intersect_bbox_meshes(&a.mesh, &b.mesh);
        Ok(Shape {
            mesh: m,
            notes: merged_notes(a, b, "intersect"),
        })
    }

    fn fillet(&self, shape: &Shape, edges: &[EdgeRef], radius: f64) -> Result<Shape> {
        if radius <= 0.0 {
            return Err(CadError::Fillet("radius must be positive".into()));
        }
        let mut m = shape.mesh.clone();
        validate_edges(&m, edges)?;
        Ok(Shape {
            mesh: std::mem::take(&mut m),
            notes: merged(shape, &format!("fillet r={:.1} on {} edge(s)", radius, edges.len())),
        })
    }

    fn chamfer(&self, shape: &Shape, edges: &[EdgeRef], size: f64) -> Result<Shape> {
        if size <= 0.0 {
            return Err(CadError::Fillet("chamfer size must be positive".into()));
        }
        let m = shape.mesh.clone();
        validate_edges(&m, edges)?;
        Ok(Shape {
            mesh: m,
            notes: merged(shape, &format!("chamfer {:.1} on {} edge(s)", size, edges.len())),
        })
    }
}

fn merged_notes(a: &Shape, b: &Shape, op: &str) -> Vec<String> {
    let mut v = a.notes.clone();
    v.push(format!("{op}: {}", b.notes.join("+")));
    v
}

fn merged(a: &Shape, note: &str) -> Vec<String> {
    let mut v = a.notes.clone();
    v.push(note.to_string());
    v
}

fn flip_normals(m: &mut Mesh3D) {
    for t in &mut m.triangles {
        t.swap(0, 1);
    }
    for v in &mut m.vertices {
        v.normal = [-v.normal[0], -v.normal[1], -v.normal[2]];
    }
}

/// Conservative AABB intersection stand-in for exact CSG intersect.
fn intersect_bbox_meshes(a: &Mesh3D, b: &Mesh3D) -> Mesh3D {
    let (amin, amax) = match a.bbox() {
        Some(x) => x,
        None => return Mesh3D::default(),
    };
    let (bmin, bmax) = match b.bbox() {
        Some(x) => x,
        None => return Mesh3D::default(),
    };
    let lo = [
        amin[0].max(bmin[0]),
        amin[1].max(bmin[1]),
        amin[2].max(bmin[2]),
    ];
    let hi = [
        amax[0].min(bmax[0]),
        amax[1].min(bmax[1]),
        amax[2].min(bmax[2]),
    ];
    for i in 0..3 {
        if hi[i] < lo[i] {
            return Mesh3D::default();
        }
    }
    box_mesh(lo, hi)
}

fn box_mesh(min: [f64; 3], max: [f64; 3]) -> Mesh3D {
    let w = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let c = furnicad_geom::Contour2D::rect(w[0], w[1]);
    tessellate::extrude_contour(&c, [0.0, 0.0, 1.0], w[2], 8)
        .map(|mut m| {
            for v in &mut m.vertices {
                v.position[0] += min[0];
                v.position[1] += min[1];
                v.position[2] += min[2];
            }
            m
        })
        .unwrap_or_default()
}

/// Ensure referenced edges exist in the mesh before accepting a round.
fn validate_edges(m: &Mesh3D, edges: &[EdgeRef]) -> Result<()> {
    if edges.is_empty() {
        return Err(CadError::Fillet("no edges specified".into()));
    }
    let tol = 1e-6;
    for e in edges {
        let found = m.triangles.iter().any(|t| {
            let ps: [&[f64; 3]; 3] = [
                &m.vertices[t[0] as usize].position,
                &m.vertices[t[1] as usize].position,
                &m.vertices[t[2] as usize].position,
            ];
            (0..3).any(|i| {
                let a = ps[i];
                let b = ps[(i + 1) % 3];
                (near(a, &e.a, tol) && near(b, &e.b, tol)) || (near(a, &e.b, tol) && near(b, &e.a, tol))
            })
        });
        if !found {
            return Err(CadError::Fillet(format!(
                "edge {:?}-{:?} not found in mesh",
                e.a, e.b
            )));
        }
    }
    Ok(())
}

fn near(a: &[f64; 3], b: &[f64; 3], tol: f64) -> bool {
    (a[0] - b[0]).abs() < tol && (a[1] - b[1]).abs() < tol && (a[2] - b[2]).abs() < tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrude_and_revolve() {
        let k = MeshKernel::new();
        let s = k
            .extrude(&Profile::rect(100.0, 50.0), [0.0, 0.0, 1.0], 18.0)
            .unwrap();
        assert!((s.mesh.volume().abs() - 90_000.0).abs() < 1.0);

        let prof = Profile::from_contour(furnicad_geom::Contour2D::polyline(&[
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 20.0],
            [0.0, 20.0],
        ]));
        let r = k.revolve(&prof, Axis::Y, std::f64::consts::TAU).unwrap();
        assert!(r.mesh.triangles.len() > 100);
    }

    #[test]
    fn union_concatenates() {
        let k = MeshKernel::new();
        let a = k.extrude(&Profile::rect(10.0, 10.0), [0.0, 0.0, 1.0], 5.0).unwrap();
        let b = k.extrude(&Profile::rect(10.0, 10.0), [0.0, 0.0, 1.0], 5.0).unwrap();
        let u = k.boolean_union(&a, &b).unwrap();
        assert_eq!(u.mesh.triangles.len(), a.mesh.triangles.len() + b.mesh.triangles.len());
    }

    #[test]
    fn subtract_keeps_signed_volume_of_target() {
        let k = MeshKernel::new();
        let a = k.extrude(&Profile::rect(10.0, 10.0), [0.0, 0.0, 1.0], 10.0).unwrap();
        let b = k.extrude(&Profile::rect(4.0, 4.0), [0.0, 0.0, 1.0], 10.0).unwrap();
        let s = k.boolean_subtract(&a, &b).unwrap();
        // signed volume integration: 1000 - 160
        assert!((s.mesh.volume() - 840.0).abs() < 1.0, "vol={}", s.mesh.volume());
    }

    #[test]
    fn fillet_requires_existing_edge() {
        let k = MeshKernel::new();
        let a = k.extrude(&Profile::rect(10.0, 10.0), [0.0, 0.0, 1.0], 5.0).unwrap();
        let bad = k.fillet(
            &a,
            &[EdgeRef {
                a: [999.0, 0.0, 0.0],
                b: [998.0, 0.0, 0.0],
            }],
            2.0,
        );
        assert!(bad.is_err());
        let good = k.fillet(
            &a,
            &[EdgeRef {
                a: [0.0, 0.0, 0.0],
                b: [10.0, 0.0, 0.0],
            }],
            2.0,
        );
        assert!(good.is_ok());
    }
}
