//! Closed 2D contours built from line segments and arcs — the profile of any
//! furniture panel.

use crate::curve::{Arc2D, LineSeg2D};
use furnicad_core::error::{CadError, Result};
use serde::{Deserialize, Serialize};

/// One element of a contour.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Segment2D {
    Line(LineSeg2D),
    Arc(Arc2D),
}

impl Segment2D {
    pub fn start(&self) -> [f64; 2] {
        match self {
            Segment2D::Line(l) => l.a,
            Segment2D::Arc(a) => a.point_at(a.start_angle),
        }
    }

    pub fn end(&self) -> [f64; 2] {
        match self {
            Segment2D::Line(l) => l.b,
            Segment2D::Arc(a) => a.point_at(a.end_angle),
        }
    }

    pub fn length(&self) -> f64 {
        match self {
            Segment2D::Line(l) => l.length(),
            Segment2D::Arc(a) => a.length(),
        }
    }
}

/// A closed planar profile. Segments must chain head-to-tail and close.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contour2D {
    pub segments: Vec<Segment2D>,
}

impl Contour2D {
    /// Axis-aligned rectangle with corners at (0,0) and (w,h), CCW.
    pub fn rect(w: f64, h: f64) -> Self {
        Self {
            segments: vec![
                Segment2D::Line(LineSeg2D::new([0.0, 0.0], [w, 0.0])),
                Segment2D::Line(LineSeg2D::new([w, 0.0], [w, h])),
                Segment2D::Line(LineSeg2D::new([w, h], [0.0, h])),
                Segment2D::Line(LineSeg2D::new([0.0, h], [0.0, 0.0])),
            ],
        }
    }

    /// Rectangle with rounded corners of `radius`.
    pub fn rounded_rect(w: f64, h: f64, radius: f64) -> Self {
        Self::rounded_rect_impl(w, h, radius)
    }

    fn rounded_rect_impl(w: f64, h: f64, r: f64) -> Self {
        use std::f64::consts::FRAC_PI_2;
        let r = r.min(w / 2.0).min(h / 2.0);
        let mut segs = Vec::new();
        // bottom edge + corner at (w-r? ) ... build CCW starting after bottom-left arc
        segs.push(Segment2D::Line(LineSeg2D::new([r, 0.0], [w - r, 0.0])));
        segs.push(Segment2D::Arc(Arc2D::new([w - r, r], r, -FRAC_PI_2, 0.0, true)));
        segs.push(Segment2D::Line(LineSeg2D::new([w, r], [w, h - r])));
        segs.push(Segment2D::Arc(Arc2D::new([w - r, h - r], r, 0.0, FRAC_PI_2, true)));
        segs.push(Segment2D::Line(LineSeg2D::new([w - r, h], [r, h])));
        segs.push(Segment2D::Arc(Arc2D::new([r, h - r], r, FRAC_PI_2, FRAC_PI_2 * 2.0, true)));
        segs.push(Segment2D::Line(LineSeg2D::new([0.0, h - r], [0.0, r])));
        segs.push(Segment2D::Arc(Arc2D::new([r, r], r, FRAC_PI_2 * 3.0, FRAC_PI_2 * 4.0, true)));
        Contour2D { segments: segs }
    }

    /// Polygon through the given points (closed automatically), CCW assumed.
    pub fn polyline(points: &[[f64; 2]]) -> Self {
        let mut segs = Vec::new();
        for w in points.windows(2) {
            segs.push(Segment2D::Line(LineSeg2D::new(w[0], w[1])));
        }
        if points.len() > 2 {
            segs.push(Segment2D::Line(LineSeg2D::new(
                *points.last().unwrap(),
                points[0],
            )));
        }
        Contour2D { segments: segs }
    }

    /// Validate chaining/closure within tolerance.
    pub fn validate(&self, tol: f64) -> Result<()> {
        if self.segments.len() < 3 {
            return Err(CadError::InvalidContour(
                "contour needs at least 3 segments".into(),
            ));
        }
        for pair in self.segments.windows(2) {
            let a = pair[0].end();
            let b = pair[1].start();
            if dist2(a, b) > tol * tol {
                return Err(CadError::InvalidContour(format!(
                    "gap between segment ends {:?} and {:?}",
                    a, b
                )));
            }
        }
        let a = self.segments.last().unwrap().end();
        let b = self.segments[0].start();
        if dist2(a, b) > tol * tol {
            return Err(CadError::InvalidContour("contour not closed".into()));
        }
        Ok(())
    }

    /// Total perimeter.
    pub fn perimeter(&self) -> f64 {
        self.segments.iter().map(|s| s.length()).sum()
    }

    /// Signed area via Green's theorem (lines exact, arcs approximated by
    /// sampled chords). Positive for CCW contours.
    pub fn signed_area(&self) -> f64 {
        let poly = self.to_polyline(64);
        let mut a = 0.0;
        for w in poly.windows(2) {
            a += w[0][0] * w[1][1] - w[1][0] * w[0][1];
        }
        let n = poly.len();
        if n > 1 {
            a += poly[n - 1][0] * poly[0][1] - poly[0][0] * poly[n - 1][1];
        }
        a / 2.0
    }

    pub fn area(&self) -> f64 {
        self.signed_area().abs()
    }

    /// Flatten arcs into polyline vertices (adaptive-ish: `segments_per_rev`).
    pub fn to_polyline(&self, max_chord_segments: usize) -> Vec<[f64; 2]> {
        let mut out = Vec::new();
        for seg in &self.segments {
            match seg {
                Segment2D::Line(l) => {
                    if out.last().map(|p| dist2(*p, l.a) < 1e-18).unwrap_or(true) {
                        out.push(l.a);
                    }
                    out.push(l.b);
                }
                Segment2D::Arc(a) => {
                    let n = ((a.sweep() / std::f64::consts::TAU) * max_chord_segments as f64)
                        .ceil() as usize
                        .max(2)
                        .min(max_chord_segments);
                    let pts = a.sample(n);
                    if out.last().map(|p| dist2(*p, pts[0]) < 1e-18).unwrap_or(true) {
                        out.push(pts[0]);
                    }
                    out.extend_from_slice(&pts[1..]);
                }
            }
        }
        // remove duplicated closing vertex
        if out.len() > 1 && dist2(out[0], *out.last().unwrap()) < 1e-18 {
            out.pop();
        }
        out
    }

    /// Axis-aligned bounding box (min, max).
    pub fn bbox(&self) -> ([f64; 2], [f64; 2]) {
        let poly = self.to_polyline(128);
        let mut min = [f64::MAX, f64::MAX];
        let mut max = [f64::MIN, f64::MIN];
        for p in poly {
            min[0] = min[0].min(p[0]);
            min[1] = min[1].min(p[1]);
            max[0] = max[0].max(p[0]);
            max[1] = max[1].max(p[1]);
        }
        (min, max)
    }

    /// Is point strictly inside (ray casting on flattened polyline)?
    pub fn contains_point(&self, p: [f64; 2]) -> bool {
        let poly = self.to_polyline(256);
        let mut inside = false;
        let n = poly.len();
        for i in 0..n {
            let a = poly[i];
            let b = poly[(i + 1) % n];
            if (a[1] > p[1]) != (b[1] > p[1]) {
                let xint = a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
                if p[0] < xint {
                    inside = !inside;
                }
            }
        }
        inside
    }
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_area_perimeter() {
        let c = Contour2D::rect(600.0, 720.0);
        assert!((c.area() - 600.0 * 720.0).abs() < 1e-6);
        assert!((c.perimeter() - 2.0 * (600.0 + 720.0)).abs() < 1e-9);
        c.validate(1e-6).unwrap();
    }

    #[test]
    fn rounded_rect_validate_and_area() {
        let c = Contour2D::rounded_rect(100.0, 50.0, 10.0);
        c.validate(1e-6).unwrap();
        // slightly less than plain rect area
        let a = c.area();
        assert!(a < 5000.0 && a > 4600.0, "area={a}");
    }

    #[test]
    fn gap_detected() {
        let mut c = Contour2D::rect(10.0, 10.0);
        if let Segment2D::Line(ref mut l) = c.segments[1] {
            l.a = [11.0, 0.0];
        }
        assert!(c.validate(1e-6).is_err());
    }

    #[test]
    fn point_in_rect() {
        let c = Contour2D::rect(10.0, 10.0);
        assert!(c.contains_point([5.0, 5.0]));
        assert!(!c.contains_point([15.0, 5.0]));
    }
}
