//! Polygon offsetting (inward/outward) used for edge-band sizing, groove
//! paths and contour booleans groundwork. Operates on flattened polylines.

use furnicad_core::error::{CadError, Result};

/// Offset a closed polyline by `d` (positive = outward for CCW input) using
/// the miter method with a miter limit to avoid spikes at sharp corners.
pub fn offset_polygon(poly: &[[f64; 2]], d: f64) -> Result<Vec<[f64; 2]>> {
    let n = poly.len();
    if n < 3 {
        return Err(CadError::InvalidContour("need >= 3 points".into()));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let prev = poly[(i + n - 1) % n];
        let cur = poly[i];
        let next = poly[(i + 1) % n];
        let n1 = left_normal(prev, cur);
        let n2 = left_normal(cur, next);
        let bx = n1[0] + n2[0];
        let by = n1[1] + n2[1];
        let bl = (bx * bx + by * by).sqrt();
        if bl < 1e-12 {
            // ~180° reversal: fall back to single normal
            out.push([cur[0] + n1[0] * d, cur[1] + n1[1] * d]);
            continue;
        }
        let cos_half_sq = (n1[0] * n2[0] + n1[1] * n2[1] + 1.0) / 2.0;
        let scale = (1.0 / cos_half_sq.sqrt().max(0.25)).min(4.0); // miter limit
        out.push([cur[0] + bx / bl * d * scale, cur[1] + by / bl * d * scale]);
    }
    Ok(out)
}

fn left_normal(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let l = dx.hypot(dy);
    if l < 1e-15 {
        [0.0, 0.0]
    } else {
        [-dy / l, dx / l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrink_rect() {
        let sq = vec![[0., 0.], [10., 0.], [10., 10.], [0., 10.]];
        let o = offset_polygon(&sq, -1.0).unwrap();
        let mut a = 0.0;
        let n = o.len();
        for i in 0..n {
            let p = o[i];
            let q = o[(i + 1) % n];
            a += p[0] * q[1] - q[0] * p[1];
        }
        let area = (a / 2.0).abs();
        assert!((area - 64.0).abs() < 1e-6, "area={area}");
    }

    #[test]
    fn grow_rect() {
        let sq = vec![[0., 0.], [10., 0.], [10., 10.], [0., 10.]];
        let o = offset_polygon(&sq, 2.0).unwrap();
        let mut a = 0.0;
        let n = o.len();
        for i in 0..n {
            let p = o[i];
            let q = o[(i + 1) % n];
            a += p[0] * q[1] - q[0] * p[1];
        }
        let area = (a / 2.0).abs();
        assert!((area - 196.0).abs() < 1e-6, "area={area}");
    }
}
