//! Entity model: the in-memory store of every object in a document.
//!
//! Panels, hardware instances, holes and auxiliary geometry all live in one
//! [`EntityStore`] keyed by [`EntityId`]. Feature-specific crates attach their
//! own typed data through [`EntityKind`].

use crate::ids::{EntityId, HoleId, PanelId};
use serde::{Deserialize, Serialize};

/// Material definition stored in the document material library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    /// Board type family.
    pub kind: MaterialKind,
    /// Base color as sRGB 0xRRGGBB.
    pub color: u32,
    /// Optional texture file (relative path inside project assets).
    pub texture: Option<String>,
    /// Standard sheet size (mm) used for nesting/cutting lists.
    pub sheet: Option<(f64, f64)>,
    /// Grain direction angle in degrees (0 = along local X of panel face).
    pub grain_deg: f64,
}

/// Family of board materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialKind {
    Chipboard, // ЛДСП
    Mdf,       // МДФ
    Plywood,   // Фанера
    SolidWood,
    Plastic,
    Metal,
    Glass,
    Mirror,
}

impl std::fmt::Display for MaterialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MaterialKind::Chipboard => "ЛДСП",
            MaterialKind::Mdf => "МДФ",
            MaterialKind::Plywood => "Фанера",
            MaterialKind::SolidWood => "Массив",
            MaterialKind::Plastic => "Пластик",
            MaterialKind::Metal => "Металл",
            MaterialKind::Glass => "Стекло",
            MaterialKind::Mirror => "Зеркало",
        };
        f.write_str(s)
    }
}

/// A named material library entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialLibrary {
    items: Vec<(crate::ids::MaterialId, Material)>,
}

impl MaterialLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a material, returning its id.
    pub fn add(&mut self, m: Material) -> crate::ids::MaterialId {
        let id = crate::ids::MaterialId(self.items.len() as u64 + 1);
        self.items.push((id, m));
        id
    }

    pub fn get(&self, id: crate::ids::MaterialId) -> Option<&Material> {
        self.items.iter().find(|(i, _)| *i == id).map(|(_, m)| m)
    }

    pub fn iter(&self) -> impl Iterator<Item = (crate::ids::MaterialId, &Material)> {
        self.items.iter().map(|(i, m)| (*i, m))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Metadata attached to entities (tags, notes, custom properties).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub tags: Vec<String>,
    pub note: String,
    pub custom: Vec<(String, String)>,
}

/// Lightweight reference to a hole produced by automatic drilling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoleRef {
    pub id: HoleId,
    pub panel: PanelId,
}

/// Generic entity payload. Feature crates define the concrete structs; core
/// keeps only cross-cutting variants so that `EntityKind` remains serializable
/// without depending on higher layers. Higher-level crates extend the document
/// with their own typed stores and use [`EntityRecord`] bookkeeping here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityPayload {
    Assembly {
        id: crate::ids::AssemblyId,
        name: String,
        children: Vec<EntityId>,
    },
    Panel {
        id: PanelId,
        name: String,
    },
    BentPanel {
        id: PanelId,
        name: String,
    },
    HardwareInstance {
        id: crate::ids::HardwareInstanceId,
        hardware: crate::ids::HardwareId,
        name: String,
        panel: Option<PanelId>,
    },
    Hole(HoleRef),
    Groove {
        id: crate::ids::GrooveId,
        panel: PanelId,
    },
    Cutout {
        id: crate::ids::CutoutId,
        panel: PanelId,
    },
    EdgeBand {
        id: crate::ids::EdgeBandId,
        panel: PanelId,
    },
    AuxiliaryPlane {
        id: crate::ids::PlaneId,
        origin: [f64; 3],
        normal: [f64; 3],
    },
    AuxiliaryLine {
        id: crate::ids::LineId,
        a: [f64; 3],
        b: [f64; 3],
    },
    Dimension {
        id: crate::ids::DimensionId,
        label: String,
    },
    Sketch {
        id: crate::ids::SketchId,
        name: String,
    },
}

/// Bookkeeping record for any entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRecord {
    pub id: EntityId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub metadata: Metadata,
    pub payload: EntityPayload,
}

/// Sparse, stable-id storage of all entities in a document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityStore {
    records: Vec<EntityRecord>,
    next: u64,
}

impl EntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a new entity and return its assigned id.
    pub fn insert(&mut self, name: impl Into<String>, payload: EntityPayload) -> EntityId {
        self.next += 1;
        let id = EntityId(self.next);
        self.records.push(EntityRecord {
            id,
            name: name.into(),
            visible: true,
            locked: false,
            metadata: Metadata::default(),
            payload,
        });
        id
    }

    pub fn get(&self, id: EntityId) -> Option<&EntityRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut EntityRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    /// Remove an entity; returns `true` if it existed.
    pub fn remove(&mut self, id: EntityId) -> bool {
        let before = self.records.len();
        self.records.retain(|r| r.id != id);
        self.records.len() != before
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntityRecord> {
        self.records.iter()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// All panels (including bent panels) in insertion order.
    pub fn panels(&self) -> Vec<&EntityRecord> {
        self.records
            .iter()
            .filter(|r| {
                matches!(
                    r.payload,
                    EntityPayload::Panel { .. } | EntityPayload::BentPanel { .. }
                )
            })
            .collect()
    }
}

/// User selection state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelectionSet {
    pub entities: Vec<EntityId>,
}

impl SelectionSet {
    pub fn select(&mut self, id: EntityId) {
        if !self.entities.contains(&id) {
            self.entities.push(id);
        }
    }

    pub fn deselect(&mut self, id: EntityId) {
        self.entities.retain(|e| *e != id);
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains(&id)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_remove() {
        let mut store = EntityStore::new();
        let id = store.insert(
            "Боковина левая",
            EntityPayload::Panel {
                id: PanelId(1),
                name: "Боковина левая".into(),
            },
        );
        assert_eq!(store.len(), 1);
        assert!(store.get(id).is_some());
        assert_eq!(store.panels().len(), 1);
        assert!(store.remove(id));
        assert!(store.get(id).is_none());
    }

    #[test]
    fn selection_ops() {
        let mut sel = SelectionSet::default();
        sel.select(EntityId(1));
        sel.select(EntityId(1));
        sel.select(EntityId(2));
        assert_eq!(sel.len(), 2);
        sel.deselect(EntityId(1));
        assert!(!sel.contains(EntityId(1)));
    }

    #[test]
    fn material_library() {
        let mut lib = MaterialLibrary::new();
        let id = lib.add(Material {
            name: "Дуб Сонома".into(),
            kind: MaterialKind::Chipboard,
            color: 0xC8A16B,
            texture: Some("oak_sonoma.jpg".into()),
            sheet: Some((2750.0, 1830.0)),
            grain_deg: 0.0,
        });
        assert_eq!(lib.get(id).unwrap().name, "Дуб Сонома");
    }
}
