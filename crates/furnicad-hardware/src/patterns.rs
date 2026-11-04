//! Hole patterns and automatic drilling generation (spec §10).

use furnicad_core::{Expr, HoleId, PanelId};
use furnicad_panels::panel::{Hole, HoleDepth, HoleType, PanelRole};
use serde::{Deserialize, Serialize};

/// One hole of a pattern, positioned relative to the pattern anchor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoleTemplate {
    pub dx: f64,
    pub dy: f64,
    pub axis: [f64; 3],
    pub diameter: f64,
    pub depth: HoleDepth,
    pub hole_type: HoleType,
    pub from_front: bool,
}

/// Reference frame for placing a pattern on a panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternReference {
    /// Anchor at bottom-left corner of the face.
    FaceCorner,
    /// Anchor centered on the face.
    FaceCenter,
    /// Anchor on an edge (used for confirmats/dowels into the thickness).
    EdgeCenter,
}

/// Named parametric set of hole templates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolePattern {
    pub name: String,
    pub holes: Vec<HoleTemplate>,
    pub reference: PatternReference,
    pub parameterized: bool,
}

impl HolePattern {
    pub fn empty(name: impl Into<String>) -> Self {
        Self { name: name.into(), holes: vec![], reference: PatternReference::FaceCorner, parameterized: false }
    }
    pub fn single(name: impl Into<String>, t: HoleTemplate) -> Self {
        Self { name: name.into(), holes: vec![t], reference: PatternReference::EdgeCenter, parameterized: false }
    }
    pub fn from_templates(name: impl Into<String>, first: HoleTemplate, rest: Vec<HoleTemplate>) -> Self {
        let mut holes = vec![first];
        holes.extend(rest);
        Self { name: name.into(), holes, reference: PatternReference::FaceCorner, parameterized: true }
    }
    pub fn all_templates(&self) -> &[HoleTemplate] {
        &self.holes
    }
}

/// How the fitting position is derived from the panel geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PositionMode {
    /// Fixed distance from edges (e.g., handle center).
    FromEdges { x_from_left: Expr, y_from_bottom: Expr },
    /// Evenly distributed along one axis with end margins (hinges).
    Distributed { count: u32, margin_start: f64, margin_end: f64 },
    /// Centered.
    Centered,
}

/// Mounting rule binding a hardware definition to a panel role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MountingRule {
    pub target_role: PanelRole,
    pub pattern_name: String,
    pub position_mode: PositionMode,
}

/// Generate concrete [`Hole`]s for a hinge/handle placement on `panel`.
///
/// `count` instances are distributed along the vertical axis when the pattern
/// is `Distributed`; otherwise a single anchored instance is produced.
pub fn generate_holes(
    panel: &furnicad_panels::panel::Panel,
    pattern: &HolePattern,
    mode: &PositionMode,
    panel_thickness: f64,
    next_hole_id: &mut u64,
) -> Vec<Hole> {
    let (w, h) = panel.size();
    let anchors: Vec<(f64, f64)> = match mode {
        PositionMode::FromEdges { x_from_left, y_from_bottom } => {
            let x = match x_from_left { Expr::Const(v) => *v, _ => 0.0 };
            let y = match y_from_bottom { Expr::Const(v) => *v, _ => 0.0 };
            vec![(x, y)]
        }
        PositionMode::Centered => vec![(w / 2.0, h / 2.0)],
        PositionMode::Distributed { count, margin_start, margin_end } => {
            let n = (*count).max(1) as usize;
            if n == 1 {
                vec![(w / 2.0, h / 2.0)]
            } else {
                let span = (h - margin_start - margin_end).max(0.0);
                (0..n)
                    .map(|i| (w / 2.0, margin_start + span * i as f64 / (n - 1) as f64))
                    .collect()
            }
        }
    };

    let mut out = Vec::new();
    for (ax, ay) in anchors {
        for t in &pattern.holes {
            *next_hole_id += 1;
            let z = if t.from_front { 0.0 } else { panel_thickness };
            out.push(Hole {
                id: HoleId(*next_hole_id),
                panel_id: panel.id,
                position: [ax + t.dx, ay + t.dy, z],
                axis: t.axis,
                diameter: Expr::constant(t.diameter),
                depth: t.depth,
                hole_type: t.hole_type,
                from_front: t.from_front,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use furnicad_core::MaterialId;
    use furnicad_panels::panel::Panel;

    #[test]
    fn hinge_pattern_distributed_three() {
        let panel = Panel::rect(PanelId(1), "Фасад", PanelRole::Front, MaterialId(1), 300.0, 700.0, 18.0, [0.0; 3]);
        let lib = crate::library::default_library();
        let hinge = lib.iter().find(|d| d.id == furnicad_core::HardwareId(1)).unwrap();
        let mut nid = 0u64;
        let holes = generate_holes(
            &panel,
            &hinge.hole_pattern,
            &PositionMode::Distributed { count: 3, margin_start: 100.0, margin_end: 100.0 },
            18.0,
            &mut nid,
        );
        // 3 anchors × 3 templates
        assert_eq!(holes.len(), 9);
        assert!(holes.iter().all(|ho| ho.position[0] > 0.0 && ho.position[1] > 0.0));
        // cups are 35 mm
        assert!(holes.iter().filter(|ho| ho.hole_type == HoleType::HingeCup).count() == 3);
    }
}
