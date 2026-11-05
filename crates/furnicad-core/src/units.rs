//! Units handling. Millimeters are the primary unit for furniture design.

use serde::{Deserialize, Serialize};

/// Length unit used throughout the document. Internally every length is stored
/// as a `f64` in [`Unit::Millimeter`]; other units exist for display/import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
}

impl Unit {
    /// Factor to multiply a value expressed in `self` to obtain millimeters.
    pub fn to_mm_factor(self) -> f64 {
        match self {
            Unit::Millimeter => 1.0,
            Unit::Centimeter => 10.0,
            Unit::Meter => 1000.0,
            Unit::Inch => 25.4,
        }
    }

    /// Convert a value from this unit into millimeters.
    #[inline]
    pub fn to_mm(self, value: f64) -> f64 {
        value * self.to_mm_factor()
    }

    /// Convert a value given in millimeters into this unit.
    #[inline]
    pub fn from_mm(self, mm: f64) -> f64 {
        mm / self.to_mm_factor()
    }

    /// Short label used by the UI.
    pub fn label(self) -> &'static str {
        match self {
            Unit::Millimeter => "mm",
            Unit::Centimeter => "cm",
            Unit::Meter => "m",
            Unit::Inch => "in",
        }
    }
}

impl Default for Unit {
    fn default() -> Self {
        Unit::Millimeter
    }
}

/// Geometric tolerance used for comparisons ("work with допуски").
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tolerance {
    /// Minimal distance between two points considered distinct.
    pub point: f64,
    /// Minimal angular difference (radians) considered distinct.
    pub angle: f64,
}

impl Default for Tolerance {
    fn default() -> Self {
        // 1 micron and ~0.0001 degree — plenty for furniture CAD in mm.
        Self {
            point: 1e-3,
            angle: 1e-6,
        }
    }
}

impl Tolerance {
    /// Are two scalar lengths equal within tolerance?
    #[inline]
    pub fn len_eq(&self, a: f64, b: f64) -> bool {
        (a - b).abs() <= self.point
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        assert!((Unit::Meter.to_mm(1.0) - 1000.0).abs() < 1e-12);
        assert!((Unit::Inch.to_mm(1.0) - 25.4).abs() < 1e-12);
        assert!((Unit::Centimeter.from_mm(250.0) - 25.0).abs() < 1e-12);
    }

    #[test]
    fn tolerance_check() {
        let t = Tolerance::default();
        assert!(t.len_eq(10.0, 10.0005));
        assert!(!t.len_eq(10.0, 10.01));
    }
}
