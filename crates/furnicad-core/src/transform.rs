//! Rigid-body transforms (translation/rotation/scale) in double precision.

use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

/// A 4x4 homogeneous transform. Panels and hardware instances carry one of
/// these to place local geometry into assembly space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    #[serde(with = "matrix_serde")]
    pub matrix: DMat4,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        matrix: DMat4::IDENTITY,
    };

    /// Pure translation.
    pub fn from_translation(t: DVec3) -> Self {
        Self {
            matrix: DMat4::from_translation(t),
        }
    }

    /// Rotation about an arbitrary axis by `angle_rad` radians.
    pub fn from_axis_angle(axis: DVec3, angle_rad: f64) -> Self {
        Self {
            matrix: DMat4::from_axis_angle(axis.normalize_or_zero(), angle_rad),
        }
    }

    /// Uniform scale.
    pub fn from_scale(s: f64) -> Self {
        Self {
            matrix: DMat4::from_scale(DVec3::splat(s)),
        }
    }

    /// Compose: `self * other` (apply `other` first).
    pub fn then(&self, other: &Transform) -> Transform {
        Transform {
            matrix: self.matrix * other.matrix,
        }
    }

    /// Transform a point.
    pub fn point(&self, p: DVec3) -> DVec3 {
        self.matrix.transform_point3(p)
    }

    /// Transform a direction vector (rotation/scale only).
    pub fn vector(&self, v: DVec3) -> DVec3 {
        self.matrix.transform_vector3(v)
    }

    /// Translation component.
    pub fn translation(&self) -> DVec3 {
        self.matrix.col(3).truncate()
    }

    /// Inverse (assumes rigid or uniformly-scaled transform).
    pub fn inverse(&self) -> Option<Transform> {
        let inv = self.matrix.inverse();
        // glam always returns a matrix; treat non-invertible as None via determinant check
        if self.matrix.determinant().abs() < 1e-12 {
            None
        } else {
            Some(Transform { matrix: inv })
        }
    }
}

// DMat4 has no serde support out of the box; serialize column-major f64 array.
mod matrix_serde {
    use glam::DMat4;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(m: &DMat4, s: S) -> Result<S::Ok, S::Error> {
        let cols: [[f64; 4]; 4] = [
            m.col(0).into(),
            m.col(1).into(),
            m.col(2).into(),
            m.col(3).into(),
        ];
        cols.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DMat4, D::Error> {
        let cols: [[f64; 4]; 4] = <[[f64; 4]; 4]>::deserialize(d)?;
        Ok(DMat4::from_cols_array_2d(&cols))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_point() {
        let t = Transform::from_translation(DVec3::new(10.0, 20.0, 30.0));
        let p = t.point(DVec3::new(1.0, 1.0, 1.0));
        assert_eq!(p, DVec3::new(11.0, 21.0, 31.0));
    }

    #[test]
    fn rotate_90_about_z() {
        let t = Transform::from_axis_angle(DVec3::Z, std::f64::consts::FRAC_PI_2);
        let v = t.vector(DVec3::X);
        assert!((v - DVec3::Y).length() < 1e-9);
    }

    #[test]
    fn compose_and_inverse() {
        let a = Transform::from_translation(DVec3::X * 5.0);
        let b = Transform::from_scale(2.0);
        let c = a.then(&b);
        let inv = c.inverse().unwrap();
        let p = DVec3::new(3.0, 4.0, 5.0);
        let back = inv.point(c.point(p));
        assert!((back - p).length() < 1e-9);
    }

    #[test]
    fn serde_roundtrip() {
        let t = Transform::from_translation(DVec3::new(1.0, 2.0, 3.0));
        let json = serde_json::to_string(&t).unwrap();
        let back: Transform = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }
}
