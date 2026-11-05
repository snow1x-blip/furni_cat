//! Triangle mesh container used by the renderer and exporters.

use serde::{Deserialize, Serialize};

/// One vertex: position + normal + uv.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f64; 3],
    pub normal: [f64; 3],
    pub uv: [f64; 2],
}

/// Indexed triangle mesh with flat vertex list.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Mesh3D {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<[u32; 3]>,
}

impl Mesh3D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_tri(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        let i = self.vertices.len() as u32;
        self.vertices.push(a);
        self.vertices.push(b);
        self.vertices.push(c);
        self.triangles.push([i, i + 1, i + 2]);
    }

    /// Append another mesh into this one.
    pub fn extend(&mut self, other: &Mesh3D) {
        let base = self.vertices.len() as u32;
        self.vertices.extend(other.vertices.iter().copied());
        for t in &other.triangles {
            self.triangles.push([t[0] + base, t[1] + base, t[2] + base]);
        }
    }

    /// Recompute per-face normals (flat shading).
    pub fn compute_flat_normals(&mut self) {
        for t in &self.triangles {
            let p0 = self.vertices[t[0] as usize].position;
            let p1 = self.vertices[t[1] as usize].position;
            let p2 = self.vertices[t[2] as usize].position;
            let n = normalize(cross(sub(p1, p0), sub(p2, p0)));
            for idx in t {
                self.vertices[*idx as usize].normal = n;
            }
        }
    }

    pub fn bbox(&self) -> Option<([f64; 3], [f64; 3])> {
        if self.vertices.is_empty() {
            return None;
        }
        let mut min = [f64::MAX; 3];
        let mut max = [f64::MIN; 3];
        for v in &self.vertices {
            for i in 0..3 {
                min[i] = min[i].min(v.position[i]);
                max[i] = max[i].max(v.position[i]);
            }
        }
        Some((min, max))
    }

    /// Total surface area.
    pub fn area(&self) -> f64 {
        let mut a = 0.0;
        for t in &self.triangles {
            let p0 = self.vertices[t[0] as usize].position;
            let p1 = self.vertices[t[1] as usize].position;
            let p2 = self.vertices[t[2] as usize].position;
            let c = cross(sub(p1, p0), sub(p2, p0));
            a += length(&c) * 0.5;
        }
        a
    }

    /// Closed-mesh volume via divergence theorem (signed).
    pub fn volume(&self) -> f64 {
        let mut vol = 0.0;
        for t in &self.triangles {
            let p0 = self.vertices[t[0] as usize].position;
            let p1 = self.vertices[t[1] as usize].position;
            let p2 = self.vertices[t[2] as usize].position;
            vol += dot(p0, cross(p1, p2)) / 6.0;
        }
        vol
    }
}

pub(crate) fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub(crate) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub(crate) fn length(a: &[f64; 3]) -> f64 {
    dot(*a, *a).sqrt()
}
pub(crate) fn normalize(a: [f64; 3]) -> [f64; 3] {
    let l = length(&a);
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tetra_volume_and_area() {
        let mut m = Mesh3D::new();
        let o = |p: [f64; 3]| Vertex {
            position: p,
            normal: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0],
        };
        // outward faces of tetra (0,0,0),(1,0,0),(0,1,0),(0,0,1)
        m.push_tri(o([0., 0., 0.]), o([0., 1., 0.]), o([1., 0., 0.]));
        m.push_tri(o([0., 0., 0.]), o([0., 0., 1.]), o([0., 1., 0.]));
        m.push_tri(o([0., 0., 0.]), o([1., 0., 0.]), o([0., 0., 1.]));
        m.push_tri(o([1., 0., 0.]), o([0., 1., 0.]), o([0., 0., 1.]));
        assert!((m.volume().abs() - 1.0 / 6.0).abs() < 1e-9, "vol={}", m.volume());
        assert!(m.area() > 0.0);
    }

    #[test]
    fn extend_offsets_indices() {
        let mut a = Mesh3D::new();
        let mut b = Mesh3D::new();
        let v = |p: [f64; 3]| Vertex {
            position: p,
            normal: [0.0; 3],
            uv: [0.0; 2],
        };
        a.push_tri(v([0., 0., 0.]), v([1., 0., 0.]), v([0., 1., 0.]));
        b.push_tri(v([2., 0., 0.]), v([3., 0., 0.]), v([2., 1., 0.]));
        a.extend(&b);
        assert_eq!(a.vertices.len(), 6);
        assert_eq!(a.triangles[1], [3, 4, 5]);
    }
}
