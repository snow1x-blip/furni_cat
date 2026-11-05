//! Ear-clipping triangulation of simple polygons and extrusion/revolve helpers
//! that turn 2D profiles into closed [`Mesh3D`] solids.

use crate::contour::Contour2D;
use crate::mesh::{cross, dot, length, normalize, sub, Mesh3D, Vertex};
use furnicad_core::error::{CadError, Result};

/// Triangulate a simple polygon (list of 2D points) into index triples using
/// ear clipping. O(n²) worst case — fine for panel profiles.
pub fn triangulate_polygon(poly: &[[f64; 2]]) -> Result<Vec<[usize; 3]>> {
    let n = poly.len();
    if n < 3 {
        return Err(CadError::InvalidContour("polygon needs >= 3 vertices".into()));
    }
    let mut idx: Vec<usize> = (0..n).collect();
    if signed_area_of(poly) < 0.0 {
        idx.reverse();
    }

    let mut tris = Vec::new();
    let mut guard = 0usize;
    while idx.len() > 3 && guard < 4 * n * n + 16 {
        guard += 1;
        let m = idx.len();
        let mut ear_found = false;
        for i in 0..m {
            let a = idx[(i + m - 1) % m];
            let b = idx[i];
            let c = idx[(i + 1) % m];
            let (pa, pb, pc) = (poly[a], poly[b], poly[c]);
            if cross2(pa, pb, pc) <= 0.0 {
                continue; // reflex vertex: not an ear tip
            }
            let mut ok = true;
            for &j in &idx {
                if j == a || j == b || j == c {
                    continue;
                }
                if point_in_tri(poly[j], pa, pb, pc) {
                    ok = false;
                    break;
                }
            }
            if ok {
                tris.push([a, b, c]);
                idx.remove(i);
                ear_found = true;
                break;
            }
        }
        if !ear_found {
            // Degenerate/self-intersecting polygon: fan fallback keeps app alive.
            let first = idx[0];
            for w in idx[1..].windows(2) {
                tris.push([first, w[0], w[1]]);
            }
            return Ok(tris);
        }
    }
    if idx.len() == 3 {
        tris.push([idx[0], idx[1], idx[2]]);
    }
    Ok(tris)
}

fn signed_area_of(poly: &[[f64; 2]]) -> f64 {
    let mut a = 0.0;
    let n = poly.len();
    for i in 0..n {
        let p = poly[i];
        let q = poly[(i + 1) % n];
        a += p[0] * q[1] - q[0] * p[1];
    }
    a / 2.0
}

fn cross2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn point_in_tri(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = cross2(a, b, p);
    let d2 = cross2(b, c, p);
    let d3 = cross2(c, a, p);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

/// Extrude a closed contour along `axis` by `depth`, producing a watertight
/// mesh suitable for rendering and STL/OBJ export. The contour lives in its
/// own plane spanned by (u, v); `axis` becomes the local w direction.
pub fn extrude_contour(
    contour: &Contour2D,
    axis: [f64; 3],
    depth: f64,
    chord_segments: usize,
) -> Result<Mesh3D> {
    contour.validate(1e-6)?;
    let poly = contour.to_polyline(chord_segments.max(8));
    let n = poly.len();
    if n < 3 {
        return Err(CadError::InvalidContour(
            "contour degenerates to < 3 vertices".into(),
        ));
    }
    let w = normalize(axis);
    let u = orthonormal_u(w);
    let v = cross(w, u);
    let to3 = |p: [f64; 2], off: f64| -> [f64; 3] {
        [
            p[0] * u[0] + p[1] * v[0] + w[0] * off,
            p[0] * u[1] + p[1] * v[1] + w[1] * off,
            p[0] * u[2] + p[1] * v[2] + w[2] * off,
        ]
    };

    let mut mesh = Mesh3D::new();
    let tris = triangulate_polygon(&poly)?;

    // Bottom cap at offset 0, normal -w (reversed winding).
    for t in &tris {
        let a = to3(poly[t[2]], 0.0);
        let b = to3(poly[t[1]], 0.0);
        let c = to3(poly[t[0]], 0.0);
        mesh.push_tri(ev(a, neg(w)), ev(b, neg(w)), ev(c, neg(w)));
    }
    // Top cap at `depth`, normal +w.
    for t in &tris {
        let a = to3(poly[t[0]], depth);
        let b = to3(poly[t[1]], depth);
        let c = to3(poly[t[2]], depth);
        mesh.push_tri(ev(a, w), ev(b, w), ev(c, w));
    }
    // Walls.
    for i in 0..n {
        let j = (i + 1) % n;
        let p0 = to3(poly[i], 0.0);
        let p1 = to3(poly[j], 0.0);
        let p2 = to3(poly[j], depth);
        let p3 = to3(poly[i], depth);
        let wall_n = normalize(cross(sub(p1, p0), sub(p3, p0)));
        mesh.push_tri(ev(p0, wall_n), ev(p1, wall_n), ev(p2, wall_n));
        mesh.push_tri(ev(p0, wall_n), ev(p2, wall_n), ev(p3, wall_n));
    }
    Ok(mesh)
}

/// Revolve a profile around the Y axis by `angle_rad` (full revolve when
/// `angle_rad >= 2π`). Profile points are `(x = radius, y = height)` with
/// x ≥ 0. Used for legs, balusters, handles, supports.
pub fn revolve_profile(profile: &[[f64; 2]], angle_rad: f64, steps: usize) -> Result<Mesh3D> {
    if profile.len() < 2 {
        return Err(CadError::geometry("profile needs >= 2 points"));
    }
    let full = angle_rad >= std::f64::consts::TAU - 1e-9;
    let steps = steps.max(3);
    let n = profile.len();
    let mut mesh = Mesh3D::new();
    let ring_pt = |i: usize, k: usize| -> [f64; 3] {
        let t = k as f64 / steps as f64 * if full { std::f64::consts::TAU } else { angle_rad };
        let (r, y) = (profile[i][0], profile[i][1]);
        [r * t.cos(), y, r * t.sin()]
    };
    let last = if full { steps } else { steps - 1 };
    for kk in 0..last {
        let k0 = kk % steps;
        let k1 = (kk + 1) % steps;
        for i in 0..n - 1 {
            let j = i + 1;
            let a = ring_pt(i, k0);
            let b = ring_pt(j, k0);
            let c = ring_pt(j, k1);
            let d = ring_pt(i, k1);
            let nn = normalize(cross(sub(b, a), sub(d, a)));
            mesh.push_tri(ev(a, nn), ev(b, nn), ev(c, nn));
            mesh.push_tri(ev(a, nn), ev(c, nn), ev(d, nn));
        }
    }
    if !full {
        // Flat end membranes approximated by fans from ring centroid.
        for &(k_idx, front) in &[(0usize, true), (steps, false)] {
            let ring: Vec<[f64; 3]> = (0..n).map(|i| ring_pt(i, k_idx)).collect();
            let cen = [
                ring.iter().map(|p| p[0]).sum::<f64>() / n as f64,
                ring.iter().map(|p| p[1]).sum::<f64>() / n as f64,
                ring.iter().map(|p| p[2]).sum::<f64>() / n as f64,
            ];
            for i in 0..ring.len() - 1 {
                let a = ring[i];
                let b = ring[i + 1];
                let nn = normalize(cross(sub(b, a), sub(cen, a)));
                if front {
                    mesh.push_tri(ev(a, nn), ev(b, nn), ev(cen, nn));
                } else {
                    mesh.push_tri(ev(b, nn), ev(a, nn), ev(cen, nn));
                }
            }
        }
    }
    Ok(mesh)
}

fn ev(p: [f64; 3], n: [f64; 3]) -> Vertex {
    Vertex {
        position: p,
        normal: n,
        uv: [0.0, 0.0],
    }
}
fn neg(a: [f64; 3]) -> [f64; 3] {
    [-a[0], -a[1], -a[2]]
}
fn orthonormal_u(w: [f64; 3]) -> [f64; 3] {
    let refv = if w[2].abs() < 0.9 { [0., 0., 1.] } else { [1., 0., 0.] };
    normalize(cross(refv, w))
}

// Silence unused-import warnings when helper fns are only cfg-test used.
#[allow(dead_code)]
fn _unused(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b) + length(&a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulate_square() {
        let sq = vec![[0., 0.], [10., 0.], [10., 10.], [0., 10.]];
        let tris = triangulate_polygon(&sq).unwrap();
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn extruded_box_is_watertight() {
        let c = Contour2D::rect(100.0, 50.0);
        let m = extrude_contour(&c, [0., 0., 1.], 18.0, 16).unwrap();
        assert!((m.volume().abs() - 90_000.0) < 1.0, "vol={}", m.volume());
    }

    #[test]
    fn revolved_cylinder_volume() {
        let prof = vec![[0., 0.], [10., 0.], [10., 20.], [0., 20.]];
        let m = revolve_profile(&prof, std::f64::consts::TAU, 64).unwrap();
        let expected = std::f64::consts::PI * 100.0 * 20.0;
        let vol = m.volume().abs();
        // polygonal approximation slightly underestimates a true cylinder
        assert!(
            vol < expected && vol > expected * 0.98,
            "vol={vol} exp={expected}"
        );
    }
}
