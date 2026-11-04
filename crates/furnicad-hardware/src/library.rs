//! Hardware definitions and the built-in library (hinge, handle, confirmat,
//! dowel, minifix, runner, leg).

use furnicad_core::HardwareId;
use serde::{Deserialize, Serialize};

/// Functional category of a hardware item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareCategory {
    Hinge,
    Handle,
    Runner,     // направляющие
    Leg,        // опора/ножка
    Lift,       // подъемный механизм
    Confirmat,  // конфирмат 7×50
    Eccentric,  // эксцентрик
    Dowel,      // шкант
    Minifix,
    Rafix,
    Custom,
}

impl std::fmt::Display for HardwareCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HardwareCategory::Hinge => "Петля",
            HardwareCategory::Handle => "Ручка",
            HardwareCategory::Runner => "Направляющие",
            HardwareCategory::Leg => "Опора",
            HardwareCategory::Lift => "Подъемник",
            HardwareCategory::Confirmat => "Конфирмат",
            HardwareCategory::Eccentric => "Эксцентрик",
            HardwareCategory::Dowel => "Шкант",
            HardwareCategory::Minifix => "Минификс",
            HardwareCategory::Rafix => "Рафикс",
            HardwareCategory::Custom => "Пользовательская",
        };
        f.write_str(s)
    }
}

/// Simplified display geometry of the fitting itself (cylinder/box list).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HardwareGeometry {
    Cylinder { diameter: f64, height: f64 },
    Box { size: [f64; 3] },
    Profile { length: f64, section: Vec<[f64; 2]> },
    None,
}

/// Editable parameter of a definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareParameter {
    pub name: String,
    pub value: f64,
    pub unit: &'static str,
}

/// A parametric hardware catalogue entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareDefinition {
    pub id: HardwareId,
    pub name: String,
    pub category: HardwareCategory,
    pub geometry: HardwareGeometry,
    pub hole_pattern: crate::patterns::HolePattern,
    pub parameters: Vec<HardwareParameter>,
}

/// A placed instance in a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareInstance {
    pub id: furnicad_core::HardwareInstanceId,
    pub definition: HardwareId,
    pub panel: furnicad_core::PanelId,
    /// Local position on the panel face.
    pub position: [f64; 3],
    pub rotation_deg: f64,
    pub params: Vec<(String, f64)>,
}

/// Built-in starter library used when no user catalogue is loaded.
pub fn default_library() -> Vec<HardwareDefinition> {
    use crate::patterns::{HolePattern, HoleTemplate};
    use furnicad_panels::panel::{HoleDepth, HoleType};

    let hinge_cup = HoleTemplate {
        dx: 0.0,
        dy: 0.0,
        axis: [0.0, 0.0, 1.0],
        diameter: 35.0,
        depth: HoleDepth::Blind(13.0),
        hole_type: HoleType::HingeCup,
        from_front: true,
    };
    let hinge_screws = vec![
        HoleTemplate { dx: -21.0, dy: 0.0, axis: [0.0, 0.0, 1.0], diameter: 4.0, depth: HoleDepth::Blind(10.0), hole_type: HoleType::Blind, from_front: true },
        HoleTemplate { dx: 21.0, dy: 0.0, axis: [0.0, 0.0, 1.0], diameter: 4.0, depth: HoleDepth::Blind(10.0), hole_type: HoleType::Blind, from_front: true },
    ];

    vec![
        HardwareDefinition {
            id: HardwareId(1),
            name: "Петля накладная 35 мм".into(),
            category: HardwareCategory::Hinge,
            geometry: HardwareGeometry::Cylinder { diameter: 35.0, height: 11.0 },
            hole_pattern: HolePattern::from_templates("Чашка + шурупы", hinge_cup.clone(), hinge_screws),
            parameters: vec![
                HardwareParameter { name: "overlay".into(), value: 16.0, unit: "mm" },
                HardwareParameter { name: "opening_angle".into(), value: 110.0, unit: "deg" },
            ],
        },
        HardwareDefinition {
            id: HardwareId(2),
            name: "Ручка-скоба 96 мм (Mepla)".into(),
            category: HardwareCategory::Handle,
            geometry: HardwareGeometry::Box { size: [108.0, 24.0, 30.0] },
            hole_pattern: HolePattern::from_templates(
                "2 отверстия 96 мм",
                HoleTemplate { dx: -48.0, dy: 0.0, axis: [0.0, 0.0, 1.0], diameter: 4.5, depth: HoleDepth::Through, hole_type: HoleType::Handle, from_front: false },
                vec![HoleTemplate { dx: 48.0, dy: 0.0, axis: [0.0, 0.0, 1.0], diameter: 4.5, depth: HoleDepth::Through, hole_type: HoleType::Handle, from_front: false }],
            ),
            parameters: vec![HardwareParameter { name: "center_distance".into(), value: 96.0, unit: "mm" }],
        },
        HardwareDefinition {
            id: HardwareId(3),
            name: "Конфирмат 7×50".into(),
            category: HardwareCategory::Confirmat,
            geometry: HardwareGeometry::Cylinder { diameter: 7.0, height: 50.0 },
            hole_pattern: HolePattern::single(
                "Стандартная присадка конфирмата",
                HoleTemplate { dx: 0.0, dy: 0.0, axis: [1.0, 0.0, 0.0], diameter: 7.0, depth: HoleDepth::Blind(50.0), hole_type: HoleType::Confirmat, from_front: false },
            ),
            parameters: vec![],
        },
        HardwareDefinition {
            id: HardwareId(4),
            name: "Шкант Ø8×30".into(),
            category: HardwareCategory::Dowel,
            geometry: HardwareGeometry::Cylinder { diameter: 8.0, height: 30.0 },
            hole_pattern: HolePattern::single(
                "Глухое 15 мм с каждой стороны",
                HoleTemplate { dx: 0.0, dy: 0.0, axis: [1.0, 0.0, 0.0], diameter: 8.0, depth: HoleDepth::Blind(15.0), hole_type: HoleType::Dowel, from_front: false },
            ),
            parameters: vec![],
        },
        HardwareDefinition {
            id: HardwareId(5),
            name: "Минификс EXCENTRIC 15 мм".into(),
            category: HardwareCategory::Minifix,
            geometry: HardwareGeometry::Cylinder { diameter: 15.0, height: 11.0 },
            hole_pattern: HolePattern::from_templates(
                "Чашка + осевое",
                HoleTemplate { dx: 0.0, dy: 0.0, axis: [0.0, 0.0, 1.0], diameter: 15.0, depth: HoleDepth::Blind(11.0), hole_type: HoleType::Eccentric, from_front: true },
                vec![HoleTemplate { dx: 0.0, dy: 0.0, axis: [1.0, 0.0, 0.0], diameter: 5.0, depth: HoleDepth::Through, hole_type: HoleType::Through, from_front: false }],
            ),
            parameters: vec![],
        },
        HardwareDefinition {
            id: HardwareId(6),
            name: "Направляющие скрытого монтажа 450 мм".into(),
            category: HardwareCategory::Runner,
            geometry: HardwareGeometry::Box { size: [450.0, 13.0, 40.0] },
            hole_pattern: HolePattern::empty("Присадка по паспорту производителя"),
            parameters: vec![HardwareParameter { name: "length".into(), value: 450.0, unit: "mm" }],
        },
        HardwareDefinition {
            id: HardwareId(7),
            name: "Опора регулируемая 90–140 мм".into(),
            category: HardwareCategory::Leg,
            geometry: HardwareGeometry::Cylinder { diameter: 60.0, height: 120.0 },
            hole_pattern: HolePattern::empty("Крепление снизу"),
            parameters: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_has_hinge_with_cup() {
        let lib = default_library();
        let hinge = lib.iter().find(|d| d.category == HardwareCategory::Hinge).unwrap();
        assert_eq!(hinge.hole_pattern.all_templates().len(), 3);
    }
}
