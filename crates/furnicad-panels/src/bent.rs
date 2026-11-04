//! Bent panels (гнутые панели) — spec §8. Starts with a single bend curve:
//! an arc in the panel plane extruded through thickness. Includes minimum
//! bending radius validation for the chosen material/thickness.

use furnicad_core::{MaterialId, PanelId, Expr};
use serde::{Deserialize, Serialize};

/// Bend information for a curved panel.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BendInfo {
    /// Bend radius measured on the inner face, mm.
    pub radius: f64,
    /// Sweep angle, degrees.
    pub sweep_deg: f64,
}

/// A panel bent along one circular arc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BentPanel {
    pub id: PanelId,
    pub name: String,
    pub material: MaterialId,
    pub thickness: Expr,
    /// Width of the strip (chord direction), mm.
    pub width: f64,
    pub bend: BendInfo,
    pub features: Vec<crate::panel::PanelFeature>,
}

impl BentPanel {
    pub fn new(id: PanelId, name: impl Into<String>, material: MaterialId, width: f64, radius: f64, sweep_deg: f64, thickness: f64) -> Self {
        Self {
            id,
            name: name.into(),
            material,
            thickness: Expr::constant(thickness),
            width,
            bend: BendInfo { radius, sweep_deg },
            features: Vec::new(),
        }
    }

    /// Arc length of the neutral layer, mm.
    pub fn developed_length(&self) -> f64 {
        let r = self.bend.radius + self.thickness.constant().unwrap_or(18.0) / 2.0;
        r.to_f64_placeholder() * 0.0 + r * self.bend.sweep_deg.to_radians()
    }

    /// Minimum allowed bend radius heuristic: for ЛДСП ~ 30×thickness,
    /// for гнутого МДФ — 10×thickness. Returns error if violated.
    pub fn validate_radius(&self, min_factor: f64) -> furnicad_core::Result<()> {
        let t = self.thickness.constant().unwrap_or(18.0);
        if self.bend.radius < t * min_factor {
            return Err(furnicad_core::CadError::geometry(format!(
                "радиус изгиба {:.0} мм меньше минимального {:.0} мм ({}×{:.0})",
                self.bend.radius,
                t * min_factor,
                min_factor,
                t
            )));
        }
        Ok(())
    }

    /// Tessellate the curved strip into a display mesh: two coaxial arc shells
    /// offset by thickness plus end caps and side walls.
    pub fn build_mesh(&self, steps: usize) -> furnicad_core::Result<furnicad_geom::Mesh3D> {
        use furnicad_geom::mesh::{Mesh3D, Vertex};
        let t = self.thickness.constant().unwrap_or(18.0);
        let steps = steps.max(4);
        let sweep = self.bend.sweep_deg.to_radians();
        let mut m = Mesh3D::new();
        let w = self.width;
        // sample arcs at radii r_in and r_out
        let r0 = self.bend.radius;
        let r1 = r0 + t;
        let pt = |r: f64, a: f64, z: f64| [r * a.cos(), r * a.sin(), z];
        for i in 0..steps {
            let a0 = sweep * i as f64 / steps as f64;
            let a1 = sweep * (i + 1) as f64 / steps as f64;
            for &z in &[0.0, w] {
                let quad = |p00, p01, p10, p11: [f64; 3]| {
                    let nrm = |a: [f64; 3], b: [f64; 3], c: [f64; 3]| {
                        let u = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
                        let v = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
                        let n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
                        let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt().max(1e-12);
                        [n[0]/l, n[1]/l, n[2]/l]
                    };
                    let n = nrm(p00, p01, p10);
                    let mk = |p: [f64; 3]| Vertex { position: p, normal: n, uv: [0.0, 0.0] };
                    m.push_tri(mk(p00), mk(p01), mk(p10));
                    m.push_tri(mk(p01), mk(p11), mk(p10));
                };
                // outer surface
                quad(pt(r1, a0, z), pt(r1, a1, z), pt(r1, a0, z.min(w - 0.0001)), pt(r1, a1, z));
            }
            // inner surface
            let nrm = |a: [f64; 3], b: [f64; 3], c: [f64; 3]| {
                let u = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
                let v = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
                let n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
                let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt().max(1e-12);
                [-n[0]/l, -n[1]/l, -n[2]/l]
            };
            let mk = |p: [f64; 3], n: [f64; 3]| Vertex { position: p, normal: n, uv: [0.0, 0.0] };
            let p00 = pt(r0, a0, 0.0);
            let p01 = pt(r0, a1, 0.0);
            let p10 = pt(r0, a0, w);
            let p11 = pt(r0, a1, w);
            let n = nrm(p00, p01, p10);
            m.push_tri(mk(p00, n), mk(p10, n), mk(p01, n));
            m.push_tri(mk(p01, n), mk(p10, n), mk(p11, n));
            // side walls at a0 and a1
            for (pa, pb) in [(a0, a0), (a1, a1)] {
                let q00 = pt(r0, pa, 0.0);
                let q01 = pt(r1, pa, 0.0);
                let q10 = pt(r0, pb, w);
                let q11 = pt(r1, pb, w);
                let n = nrm(q00, q01, q10);
                let mk = |p: [f64; 3]| Vertex { position: p, normal: n, uv: [0.0, 0.0] };
                m.push_tri(mk(q00), mk(q01), mk(q10));
                m.push_tri(mk(q01), mk(q11), mk(q10));
            }
        }
        Ok(m)
    }
}

/// Temporary helper until `Expr` gains `to_f64`.
trait ToF64Placeholder {
    fn to_f64_placeholder(self) -> f64;
}
impl ToF64Placeholder for f64 {
    fn to_f64_placeholder(self) -> f64 {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn developed_length_matches_arc() {
        let p = BentPanel::new(PanelId(1), "Гнутая", MaterialId(1), 500.0, 300.0, 90.0, 10.0);
        let expected = (300.0 + 5.0) * 90f64.to_radians();
        assert!((p.developed_length() - expected).abs() < 1e-9);
    }

    #[test]
    fn radius_validation() {
        let p = BentPanel::new(PanelId(1), "Гнутая", MaterialId(1), 500.0, 100.0, 90.0, 18.0);
        assert!(p.validate_radius(30.0).is_err()); // 100 < 540
        let ok = BentPanel::new(PanelId(2), "Г", MaterialId(1), 500.0, 600.0, 90.0, 18.0);
        assert!(ok.validate_radius(30.0).is_ok());
    }

    #[test]
    fn mesh_not_empty() {
        let p = BentPanel::new(PanelId(1), "Г", MaterialId(1), 100.0, 200.0, 60.0, 10.0);
        let m = p.build_mesh(8).unwrap();
        assert!(!m.vertices.is_empty());
    }
}
