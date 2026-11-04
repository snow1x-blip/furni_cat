//! Edge bands (кромкооблицовка) — spec §9.

use furnicad_core::{EdgeBandId, EdgeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Edge band material family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeMaterial {
    None,
    Pvc,
    Abs,
    Melamine,
    Aluminum,
    SolidWood,
}

impl std::fmt::Display for EdgeMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EdgeMaterial::None => "без кромки",
            EdgeMaterial::Pvc => "ПВХ",
            EdgeMaterial::Abs => "АБС",
            EdgeMaterial::Melamine => "меламин",
            EdgeMaterial::Aluminum => "алюминиевый кант",
            EdgeMaterial::SolidWood => "массив",
        };
        f.write_str(s)
    }
}

/// Attachment method — affects machining notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeBandMethod {
    Adhesive,   // клей-расплав
    Cold,       // холодная гибкая кромка
    Mechanical, // профильный кант (Gola и т.п.)
}

/// One applied edge band.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeBand {
    pub id: EdgeBandId,
    pub edge_id: EdgeId,
    pub material: EdgeMaterial,
    pub color: u32,
    pub thickness_mm: f64,
    pub height_mm: f64,
    pub method: EdgeBandMethod,
    /// Whether the band changes the finished technological size.
    pub affects_size: bool,
    pub show_in_3d: bool,
    pub include_in_reports: bool,
}

impl EdgeBand {
    pub fn pvc(edge_id: EdgeId, thickness_mm: f64, color: u32) -> Self {
        Self {
            id: EdgeBandId(0),
            edge_id,
            material: EdgeMaterial::Pvc,
            color,
            thickness_mm,
            height_mm: thickness_mm + 2.0,
            method: EdgeBandMethod::Adhesive,
            affects_size: true,
            show_in_3d: true,
            include_in_reports: true,
        }
    }
}

/// Map of panel edges to their bands.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EdgeBandMap {
    pub edges: HashMap<u64, EdgeBand>,
    pub default_band: Option<EdgeBand>,
}

impl EdgeBandMap {
    pub fn set(&mut self, band: EdgeBand) {
        self.edges.insert(band.edge_id.raw(), band);
    }

    pub fn get(&self, edge: EdgeId) -> Option<&EdgeBand> {
        self.edges.get(&edge.raw())
    }

    /// Total band thickness contribution along one axis (for size comp).
    pub fn size_compensation(&self, edges: &[EdgeId]) -> f64 {
        edges
            .iter()
            .filter_map(|e| self.get(*e))
            .filter(|b| b.affects_size && b.material != EdgeMaterial::None)
            .map(|b| b.thickness_mm)
            .sum()
    }
}

/// Rule-based automatic banding (spec §9): visible edges get PVC 2 mm,
/// hidden edges melamine 0.4 mm; fronts get perimeter banding.
pub fn assign_default_bands(
    map: &mut EdgeBandMap,
    edges: &[EdgeId],
    visible: &[bool],
    is_front: bool,
    color: u32,
) {
    for (&e, &vis) in edges.iter().zip(visible.iter()) {
        let band = if is_front || vis {
            EdgeBand::pvc(e, 2.0, color)
        } else {
            EdgeBand {
                id: EdgeBandId(0),
                edge_id: e,
                material: EdgeMaterial::Melamine,
                color,
                thickness_mm: 0.4,
                height_mm: 0.4,
                method: EdgeBandMethod::Adhesive,
                affects_size: false,
                show_in_3d: false,
                include_in_reports: true,
            }
        };
        map.set(band);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_banding_rules() {
        let mut map = EdgeBandMap::default();
        let edges = [EdgeId(1), EdgeId(2), EdgeId(3), EdgeId(4)];
        let visible = [true, false, true, false];
        assign_default_bands(&mut map, &edges, &visible, false, 0x8B5A2B);
        assert_eq!(map.get(EdgeId(1)).unwrap().material, EdgeMaterial::Pvc);
        assert_eq!(
            map.get(EdgeId(2)).unwrap().material,
            EdgeMaterial::Melamine
        );
        assert!((map.size_compensation(&edges) - 4.0).abs() < 1e-9); // two PVC 2mm
    }
}
