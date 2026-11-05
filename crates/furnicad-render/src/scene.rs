//! Viewport scene assembly: display modes, items and overlays (spec §12).

use furnicad_geom::Mesh3D;
use serde::{Deserialize, Serialize};

/// The seven viewport display modes of the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Wireframe,
    HiddenLines,
    Shaded,
    ShadedWithEdges,
    Textured,
    XRay,
    RenderPreview,
}

impl DisplayMode {
    pub fn label(self) -> &'static str {
        match self {
            DisplayMode::Wireframe => "Каркас",
            DisplayMode::HiddenLines => "Скрытые линии",
            DisplayMode::Shaded => "Полутоновый",
            DisplayMode::ShadedWithEdges => "Шейдинг + рёбра",
            DisplayMode::Textured => "Текстуры",
            DisplayMode::XRay => "Рентген",
            DisplayMode::RenderPreview => "Рендер-превью",
        }
    }
    pub fn shows_edges(self) -> bool {
        matches!(self, DisplayMode::Wireframe | DisplayMode::ShadedWithEdges | DisplayMode::HiddenLines)
    }
    pub fn shows_faces(self) -> bool {
        !matches!(self, DisplayMode::Wireframe)
    }
}

/// One drawable object in the viewport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneItem {
    /// Entity this item belongs to (for picking/selection highlight).
    pub entity_id: u64,
    pub name: String,
    pub mesh: Mesh3D,
    /// sRGB 0xRRGGBB base color.
    pub color: u32,
    pub selected: bool,
    pub visible: bool,
}

/// Aggregated viewport content for one frame.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewportScene {
    pub items: Vec<SceneItem>,
    pub mode: DisplayMode,
    /// Overlay line segments (dimensions, grid, snapping helpers).
    pub overlay_lines: Vec<([f64; 3], [f64; 3])>,
    pub background: u32,
}

impl ViewportScene {
    pub fn add(&mut self, item: SceneItem) {
        self.items.push(item);
    }

    pub fn clear_selection(&mut self) {
        for it in &mut self.items {
            it.selected = false;
        }
    }

    pub fn select(&mut self, entity_id: u64) {
        for it in &mut self.items {
            it.selected = it.entity_id == entity_id;
        }
    }

    /// Axis-aligned bounds of all visible meshes — used for "zoom to fit" (F).
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        let mut acc: Option<([f64; 3], [f64; 3])> = None;
        for it in &self.items {
            if !it.visible {
                continue;
            }
            if let Some(b) = it.mesh.bbox() {
                acc = Some(match acc {
                    None => b,
                    Some((mn, mx)) => (
                        [mn[0].min(b.0[0]), mn[1].min(b.0[1]), mn[2].min(b.0[2])],
                        [mx[0].max(b.1[0]), mx[1].max(b.1[1]), mx[2].max(b.1[2])],
                    ),
                });
            }
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_and_bounds() {
        let mut s = ViewportScene::default();
        let mut m = Mesh3D::new();
        m.vertices.push(furnicad_geom::mesh::Vertex { position: [0.0, 0.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] });
        m.vertices.push(furnicad_geom::mesh::Vertex { position: [100.0, 50.0, 20.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] });
        s.add(SceneItem { entity_id: 1, name: "p".into(), mesh: m, color: 0, selected: false, visible: true });
        s.select(1);
        assert!(s.items[0].selected);
        let (mn, mx) = s.bounds().unwrap();
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [100.0, 50.0, 20.0]);
    }
}
