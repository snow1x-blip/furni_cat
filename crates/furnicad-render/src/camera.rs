//! Orbit camera with standard CAD views (front/top/left/axonometric).

use serde::{Deserialize, Serialize};

/// Parallel or perspective projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Projection {
    Orthographic,
    Perspective,
}

/// Named standard orientations (spec §13 "виды").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewOrientation {
    Front,
    Top,
    Left,
    Axonometric,
}

impl ViewOrientation {
    /// Direction the camera looks *from*, normalized.
    pub fn eye_dir(self) -> [f64; 3] {
        let v = match self {
            ViewOrientation::Front => [0.0, -1.0, 0.0],
            ViewOrientation::Top => [0.0, 0.0, 1.0],
            ViewOrientation::Left => [-1.0, 0.0, 0.0],
            ViewOrientation::Axonometric => [1.0, -1.0, 0.8],
        };
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    }

    pub fn up(self) -> [f64; 3] {
        match self {
            ViewOrientation::Top => [0.0, 1.0, 0.0],
            _ => [0.0, 0.0, 1.0],
        }
    }
}

/// Orbit camera around a target point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub target: [f64; 3],
    pub distance: f64,
    pub yaw_rad: f64,
    pub pitch_rad: f64,
    pub fov_deg: f64,
    pub projection: Projection,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: [300.0, 300.0, 350.0],
            distance: 1600.0,
            yaw_rad: std::f64::consts::FRAC_PI_4,
            pitch_rad: 0.35,
            fov_deg: 45.0,
            projection: Projection::Perspective,
        }
    }
}

impl Camera {
    pub fn eye(&self) -> [f64; 3] {
        let cp = self.pitch_rad.cos();
        [
            self.target[0] + self.distance * cp * self.yaw_rad.cos(),
            self.target[1] + self.distance * cp * self.yaw_rad.sin(),
            self.target[2] + self.distance * self.pitch_rad.sin(),
        ]
    }

    /// Snap to a named orientation.
    pub fn set_orientation(&mut self, o: ViewOrientation) {
        let d = o.eye_dir();
        // yaw/pitch from direction
        self.yaw_rad = d[1].atan2(d[0]);
        self.pitch_rad = (d[2]).clamp(-0.99, 0.99).asin();
    }

    /// Simple view matrix as column-major 4x4 array (OpenGL convention),
    /// computed by hand to avoid extra deps here.
    pub fn view_matrix(&self) -> [[f64; 4]; 4] {
        let eye = self.eye();
        let z = norm([eye[0] - self.target[0], eye[1] - self.target[1], eye[2] - self.target[2]]);
        let up_hint = if self.pitch_rad.abs() > 1.4 { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
        let x = norm(cross(up_hint, z));
        let y = cross(z, x);
        [
            [x[0], y[0], z[0], 0.0],
            [x[1], y[1], z[1], 0.0],
            [x[2], y[2], z[2], 0.0],
            [-dot(x, eye), -dot(y, eye), -dot(z, eye), 1.0],
        ]
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = dot(a, a).sqrt().max(1e-12);
    [a[0] / l, a[1] / l, a[2] / l]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axo_eye_is_normalized() {
        let d = ViewOrientation::Axonometric.eye_dir();
        assert!((dot(d, d) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn view_matrix_orthonormal() {
        let c = Camera::default();
        let m = c.view_matrix();
        // columns x,y,z should be unit and orthogonal
        let col = |i: usize| [m[0][i], m[1][i], m[2][i]];
        assert!((dot(col(0), col(1))).abs() < 1e-9);
        assert!((dot(col(0), col(0)) - 1.0).abs() < 1e-9);
    }
}
