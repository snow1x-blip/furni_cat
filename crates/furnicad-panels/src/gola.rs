//! Gola-profile handle cutouts for fronts (spec §8 "Вырезы под Gola").

use furnicad_core::Expr;
use serde::{Deserialize, Serialize};

/// Where a gola profile is mounted on a facade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GolaPlacement {
    HorizontalTop,
    HorizontalBottom,
    VerticalLeft,
    VerticalRight,
}

/// Parametric gola cut: profile curve approximated by points, placement edge,
/// start/end offsets and milling depth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GolaCutProfile {
    /// Profile cross-section polyline in (x, y) local milling coordinates.
    pub profile_curve: Vec<[f64; 2]>,
    pub placement: GolaPlacement,
    pub offset_start: Expr,
    pub offset_end: Expr,
    pub depth: Expr,
}

impl GolaCutProfile {
    /// Standard 21 mm gola C-profile approximation.
    pub fn standard(placement: GolaPlacement) -> Self {
        Self {
            profile_curve: vec![
                [0.0, 0.0],
                [12.0, 0.0],
                [12.0, 9.0],
                [3.0, 9.0],
                [3.0, 13.0],
                [21.0, 13.0],
                [21.0, 21.0],
                [0.0, 21.0],
            ],
            placement,
            offset_start: Expr::constant(18.0),
            offset_end: Expr::constant(18.0),
            depth: Expr::constant(13.0),
        }
    }

    /// Cut length along an edge of given total length.
    pub fn cut_length(&self, edge_len: f64, vars: &furnicad_core::VariableTable) -> furnicad_core::Result<f64> {
        let s = self.offset_start.eval(vars)?;
        let e = self.offset_end.eval(vars)?;
        let l = edge_len - s - e;
        if l <= 0.0 {
            return Err(furnicad_core::CadError::geometry("gola cut degenerates"));
        }
        Ok(l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gola_length_parametric() {
        let g = GolaCutProfile::standard(GolaPlacement::HorizontalBottom);
        let t = furnicad_core::VariableTable::new();
        assert!((g.cut_length(600.0, &t).unwrap() - 564.0).abs() < 1e-9);
    }
}
