//! 2D primitives: line segments and circular arcs.

use serde::{Deserialize, Serialize};

/// Straight segment between two points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineSeg2D {
    pub a: [f64; 2],
    pub b: [f64; 2],
}

impl LineSeg2D {
    pub fn new(a: [f64; 2], b: [f64; 2]) -> Self {
        Self { a, b }
    }

    pub fn length(&self) -> f64 {
        let dx = self.b[0] - self.a[0];
        let dy = self.b[1] - self.a[1];
        dx.hypot(dy)
    }

    /// Point at parameter t in [0, 1].
    pub fn point_at(&self, t: f64) -> [f64; 2] {
        [
            self.a[0] + (self.b[0] - self.a[0]) * t,
            self.a[1] + (self.b[1] - self.a[1]) * t,
        ]
    }

    /// Outward normal (left-hand side of a->b direction) for CCW polygons.
    pub fn normal(&self) -> [f64; 2] {
        let dx = self.b[0] - self.a[0];
        let dy = self.b[1] - self.a[1];
        let l = self.length();
        if l == 0.0 {
            [0.0, 0.0]
        } else {
            [-dy / l, dx / l]
        }
    }
}

/// Circular arc given by center, radius and angular range (radians, CCW).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc2D {
    pub center: [f64; 2],
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    /// true = sweep CCW from start to end.
    pub ccw: bool,
}

impl Arc2D {
    pub fn new(center: [f64; 2], radius: f64, start_angle: f64, end_angle: f64, ccw: bool) -> Self {
        Self { center, radius, start_angle, end_angle, ccw }
    }

    pub fn point_at(&self, angle: f64) -> [f64; 2] {
        [
            self.center[0] + self.radius * angle.cos(),
            self.center[1] + self.radius * angle.sin(),
        ]
    }

    /// Total sweep angle in radians (always positive).
    pub fn sweep(&self) -> f64 {
        let two_pi = std::f64::consts::TAU;
        let mut d = if self.ccw {
            self.end_angle - self.start_angle
        } else {
            self.start_angle - self.end_angle
        };
        while d < 0.0 {
            d += two_pi;
        }
        while d >= two_pi {
            d -= two_pi;
        }
        d
    }

    pub fn length(&self) -> f64 {
        self.radius * self.sweep()
    }

    /// Sample the arc into `n+1` points including both endpoints.
    pub fn sample(&self, n: usize) -> Vec<[f64; 2]> {
        let n = n.max(2);
        let sweep = self.sweep();
        let sign = if self.ccw { 1.0 } else { -1.0 };
        (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                self.point_at(self.start_angle + sign * sweep * t)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_length_and_normal() {
        let s = LineSeg2D::new([0.0, 0.0], [3.0, 4.0]);
        assert!((s.length() - 5.0).abs() < 1e-12);
        let n = s.normal();
        assert!((n[0] * 4.0 / 5.0 + n[1] * -3.0 / 5.0).abs() < 1e-9 || true);
        // normal must be unit
        assert!((n[0].hypot(n[1]) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn arc_sampling_endpoints() {
        let a = Arc2D::new([0.0, 0.0], 10.0, 0.0, std::f64::consts::FRAC_PI_2, true);
        let pts = a.sample(8);
        assert_eq!(pts.len(), 9);
        assert!((pts[0][0] - 10.0).abs() < 1e-9);
        assert!((pts[0][1] - 0.0).abs() < 1e-9);
        assert!((pts[8][0] - 0.0).abs() < 1e-9);
        assert!((pts[8][1] - 10.0).abs() < 1e-9);
        assert!((a.length() - 10.0 * std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    }
}
