//! `furnicad-panels` — panel technology layer.
//!
//! A panel is not just a solid: it stores its base contour, parametric
//! thickness, material, grain direction, edge bands, cutouts, grooves and
//! drilling features (spec §8). This crate also implements automatic panel
//! sizing with gaps/overlay ([`PanelCreationOptions`]), edge band maps with
//! rule-based auto-assignment, Gola-profile cutouts and bent panels along a
//! single bend curve.

pub mod band;
pub mod bent;
pub mod gola;
pub mod panel;

pub use band::{EdgeBand, EdgeBandMap, EdgeBandMethod, EdgeMaterial, assign_default_bands};
pub use bent::BentPanel;
pub use gola::GolaCutProfile;
pub use panel::{
    Groove, GrooveBottom, GrooveSection, Hole, HoleDepth, HoleType, Panel, PanelCreationOptions,
    PanelCutMethod, PanelFeature, PanelRole,
};

use furnicad_core::Result;

/// Compute the finished cut size of a facade inside a carcass opening,
/// applying gaps and optional edge compensation (spec §8 "Автоматическое
/// создание панелей").
pub fn facade_size(
    opening_w: f64,
    opening_h: f64,
    opts: &PanelCreationOptions,
) -> Result<(f64, f64)> {
    if opening_w <= 0.0 || opening_h <= 0.0 {
        return Err(furnicad_core::CadError::geometry("opening must be positive"));
    }
    let w = opening_w - opts.gap_left - opts.gap_right + 2.0 * opts.overlay;
    let h = opening_h - opts.gap_top - opts.gap_bottom + 2.0 * opts.overlay;
    if w < 10.0 || h < 10.0 {
        return Err(furnicad_core::CadError::geometry(
            "facade degenerates after gap compensation",
        ));
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_with_two_gaps() {
        let opts = PanelCreationOptions {
            gap_left: 3.0,
            gap_right: 3.0,
            ..Default::default()
        };
        let (w, _h) = facade_size(600.0, 720.0, &opts).unwrap();
        assert!((w - 594.0).abs() < 1e-9);
    }

    #[test]
    fn degenerate_opening_rejected() {
        let opts = PanelCreationOptions {
            gap_left: 100.0,
            gap_right: 100.0,
            ..Default::default()
        };
        assert!(facade_size(150.0, 720.0, &opts).is_err());
    }
}
