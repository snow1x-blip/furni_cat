//! The [`Panel`] entity and its technological features.

use furnicad_core::{EdgeId, Expr, HoleId, MaterialId, PanelId};
use furnicad_geom::Contour2D;
use serde::{Deserialize, Serialize};

/// Functional role of a panel — drives automatic edge banding and hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelRole {
    Side,      // Боковина
    TopBottom, // Дно / крыша
    Back,      // Задняя стенка
    Shelf,     // Полка
    Drawer,    // Ящик
    Front,     // Фасад
    Countertop,
    Other,
}

/// Options for automatic panel creation (spec §8).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PanelCreationOptions {
    pub gap_left: f64,
    pub gap_right: f64,
    pub gap_top: f64,
    pub gap_bottom: f64,
    /// Overlay: how much facade covers the carcass opening edge.
    pub overlay: f64,
    pub edge_compensation: bool,
    pub allow_auto_resize: bool,
}

impl Default for PanelCreationOptions {
    fn default() -> Self {
        Self {
            gap_left: 3.0,
            gap_right: 3.0,
            gap_top: 3.0,
            gap_bottom: 3.0,
            overlay: 0.0,
            edge_compensation: true,
            allow_auto_resize: true,
        }
    }
}

/// How deep a hole goes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HoleDepth {
    Through,
    Blind(f64),
}

/// Technological hole families (spec §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoleType {
    Through,
    Blind,
    Countersunk,
    Dowel,       // шкант
    Confirmat,   // конфирмат
    Eccentric,   // эксцентрик
    HingeCup,    // чашка петли 35 мм
    Lock,
    Handle,
    Minifix,
}

/// A single drilled hole in panel-local coordinates (x along width,
/// y along height, z along thickness normal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hole {
    pub id: HoleId,
    pub panel_id: PanelId,
    pub position: [f64; 3],
    pub axis: [f64; 3],
    pub diameter: Expr,
    pub depth: HoleDepth,
    pub hole_type: HoleType,
    /// Which face: front (z=0) or back (z=thickness).
    pub from_front: bool,
}

/// Groove cross-section families.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GrooveSection {
    Rect { width: f64 },
    Round { radius: f64 },
    Trapezoid { top: f64, bottom: f64 },
    Custom(Contour2D),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GrooveBottom {
    Flat,
    Rounded,
}

/// Milled groove / routing feature (spec §8 "Пазы и фрезеровки").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Groove {
    pub id: furnicad_core::GrooveId,
    pub panel_id: PanelId,
    pub path: Vec<[f64; 2]>,
    pub section: GrooveSection,
    pub depth: f64,
    pub from_front: bool,
}

/// Ways to trim a panel (spec §8 "Отсечение части панели").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PanelCutMethod {
    ByPlane { origin: [f64; 3], normal: [f64; 3] },
    ByAngle { edge: EdgeId, angle_deg: f64 },
}

/// Any feature attached to a panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PanelFeature {
    Hole(Hole),
    Groove(Groove),
    Cutout {
        id: furnicad_core::CutoutId,
        contour: Contour2D,
        through: bool,
    },
    Fillet { edge: EdgeId, radius: f64 },
    Chamfer { edge: EdgeId, size: f64 },
    Trim(PanelCutMethod),
}

/// The panel — primary object of case furniture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    pub id: PanelId,
    pub name: String,
    pub role: PanelRole,
    pub material: MaterialId,
    pub thickness: Expr,
    pub contour: Contour2D,
    /// Placement transform of the panel plane in assembly space.
    pub origin: [f64; 3],
    pub x_dir: [f64; 3],
    pub y_dir: [f64; 3],
    pub edge_bands: crate::band::EdgeBandMap,
    pub features: Vec<PanelFeature>,
    pub metadata: furnicad_core::Metadata,
}

impl Panel {
    /// Axis-aligned rectangular panel in the XY plane at `origin`.
    pub fn rect(
        id: PanelId,
        name: impl Into<String>,
        role: PanelRole,
        material: MaterialId,
        w: f64,
        h: f64,
        thickness: f64,
        origin: [f64; 3],
    ) -> Self {
        Self {
            id,
            name: name.into(),
            role,
            material,
            thickness: Expr::constant(thickness),
            contour: Contour2D::rect(w, h),
            origin,
            x_dir: [1.0, 0.0, 0.0],
            y_dir: [0.0, 1.0, 0.0],
            edge_bands: crate::band::EdgeBandMap::default(),
            features: Vec::new(),
            metadata: furnicad_core::Metadata::default(),
        }
    }

    /// Local-space extrusion direction (normal of the panel face).
    pub fn normal(&self) -> [f64; 3] {
        let (a, b) = (self.x_dir, self.y_dir);
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    /// Bounding size (w, h) of the contour in local space.
    pub fn size(&self) -> (f64, f64) {
        let ((minx, miny), (maxx, maxy)) = self.contour.bbox();
        (maxx - minx, maxy - miny)
    }

    /// Extrude the panel into a display mesh using the pure-Rust kernel.
    pub fn build_mesh(
        &self,
        thickness_mm: f64,
        chord_segments: usize,
    ) -> furnicad_core::Result<furnicad_geom::Mesh3D> {
        let mut m = furnicad_geom::tessellate::extrude_contour(
            &self.contour,
            self.normal(),
            thickness_mm,
            chord_segments,
        )?;
        // translate to panel origin
        for v in &mut m.vertices {
            v.position = [
                v.position[0] + self.origin[0],
                v.position[1] + self.origin[1],
                v.position[2] + self.origin[2],
            ];
        }
        Ok(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_panel_mesh_volume() {
        let p = Panel::rect(
            PanelId(1),
            "Боковина",
            PanelRole::Side,
            MaterialId(1),
            300.0,
            600.0,
            18.0,
            [0.0, 0.0, 0.0],
        );
        let m = p.build_mesh(18.0, 16).unwrap();
        assert!((m.volume().abs() - 300.0 * 600.0 * 18.0) < 1.0);
        assert_eq!(p.normal(), [0.0, 0.0, 1.0]);
    }
}
