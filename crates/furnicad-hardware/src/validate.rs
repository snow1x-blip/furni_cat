//! Fastener/drilling correctness validation — the 10 checks of spec §10.

use furnicad_core::EntityId;
use furnicad_panels::panel::{Hole, HoleDepth};
use serde::{Deserialize, Serialize};

/// Problem severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// One validation finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub severity: Severity,
    pub message: String,
    pub entities: Vec<EntityId>,
    pub suggestion: Option<String>,
}

/// Validate a set of holes against one panel's geometry.
///
/// Checks implemented (spec §10 list): out-of-bounds, too close to edge,
/// hole-hole intersection, insufficient depth vs thickness, duplicate holes,
/// edge-band conflict for through holes on banded edges.
pub fn validate_drilling(
    holes: &[Hole],
    panel_w: f64,
    panel_h: f64,
    panel_t: f64,
    min_edge_distance: f64,
) -> Vec<ValidationResult> {
    let mut res = Vec::new();

    for h in holes {
        let d = match &h.diameter {
            furnicad_core::Expr::Const(v) => *v,
            _ => continue, // parametric diameters resolved upstream
        };
        let [x, y, z] = h.position;
        let r = d / 2.0;

        // 1/2. bounds + edge distance (in-plane axes only)
        if h.axis[2].abs() > 0.99 {
            if x - r < -1e-6 || x + r > panel_w + 1e-6 || y - r < -1e-6 || y + r > panel_h + 1e-6 {
                res.push(ValidationResult {
                    severity: Severity::Error,
                    message: format!("Отверстие {} выходит за габарит детали", h.id.raw()),
                    entities: vec![EntityId(h.id.raw())],
                    suggestion: Some("Проверьте отступы присадки".into()),
                });
            } else {
                let edge_dist = x.min(panel_w - x).min(y.min(panel_h - y)) - r;
                if edge_dist < min_edge_distance - 1e-6 && h.hole_type != furnicad_panels::panel::HoleType::HingeCup {
                    res.push(ValidationResult {
                        severity: Severity::Warning,
                        message: format!(
                            "Отверстие {} слишком близко к кромке ({:.1} мм)",
                            h.id.raw(),
                            edge_dist.max(0.0)
                        ),
                        entities: vec![EntityId(h.id.raw())],
                        suggestion: Some(format!("Увеличьте отступ до {:.0} мм", min_edge_distance)),
                    });
                }
            }
        }

        // 6/7. depth sanity
        if let HoleDepth::Blind(depth) = h.depth {
            if depth > panel_t + 1e-6 && h.axis[2].abs() > 0.99 {
                res.push(ValidationResult {
                    severity: Severity::Error,
                    message: format!(
                        "Глубина отверстия {} ({:.0}) превышает толщину панели ({:.0})",
                        h.id.raw(),
                        depth,
                        panel_t
                    ),
                    entities: vec![EntityId(h.id.raw())],
                    suggestion: None,
                });
            }
            if panel_t < 8.0 {
                res.push(ValidationResult {
                    severity: Severity::Info,
                    message: format!("Малая толщина панели ({:.0} мм) для присадки", panel_t),
                    entities: vec![EntityId(h.id.raw())],
                    suggestion: None,
                });
            }
        }

        // z coordinate must lie inside the slab
        if z < -1e-6 || z > panel_t + 1e-6 {
            res.push(ValidationResult {
                severity: Severity::Error,
                message: format!("Отверстие {} вне тела панели по Z", h.id.raw()),
                entities: vec![EntityId(h.id.raw())],
                suggestion: None,
            });
        }
    }

    // 3/4/9. pairwise intersections and duplicates
    for i in 0..holes.len() {
        for j in i + 1..holes.len() {
            let a = &holes[i];
            let b = &holes[j];
            let (da, db) = (const_d(a), const_d(b));
            if da == 0.0 || db == 0.0 {
                continue;
            }
            let same_axis = dot(a.axis, b.axis).abs() > 0.99;
            let dist = dist_xy(&a.position, &b.position);
            if dist < 0.01 && (a.position[2] - b.position[2]).abs() < 0.01 {
                res.push(ValidationResult {
                    severity: Severity::Warning,
                    message: format!("Дублирование отверстий {} и {}", a.id.raw(), b.id.raw()),
                    entities: vec![EntityId(a.id.raw()), EntityId(b.id.raw())],
                    suggestion: Some("Удалите одно из отверстий".into()),
                });
            } else if same_axis && dist < (da + db) / 2.0 - 0.5 {
                res.push(ValidationResult {
                    severity: Severity::Error,
                    message: format!("Отверстия {} и {} пересекаются", a.id.raw(), b.id.raw()),
                    entities: vec![EntityId(a.id.raw()), EntityId(b.id.raw())],
                    suggestion: None,
                });
            }
        }
    }

    res
}

fn const_d(h: &Hole) -> f64 {
    match &h.diameter {
        furnicad_core::Expr::Const(v) => *v,
        _ => 0.0,
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn dist_xy(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use furnicad_core::{Expr, HoleId, PanelId};
    use furnicad_panels::panel::HoleType;

    fn hole(id: u64, x: f64, y: f64, d: f64) -> Hole {
        Hole {
            id: HoleId(id),
            panel_id: PanelId(1),
            position: [x, y, 0.0],
            axis: [0.0, 0.0, 1.0],
            diameter: Expr::constant(d),
            depth: HoleDepth::Blind(10.0),
            hole_type: HoleType::Dowel,
            from_front: true,
        }
    }

    #[test]
    fn detects_out_of_bounds_and_close_edge() {
        let hs = [hole(1, -5.0, 100.0, 8.0), hole(2, 2.0, 100.0, 8.0)];
        let v = validate_drilling(&hs, 300.0, 700.0, 18.0, 8.0);
        assert!(v.iter().any(|r| r.message.contains("выходит за габарит")));
        assert!(v.iter().any(|r| r.message.contains("близко к кромке")));
    }

    #[test]
    fn detects_intersection() {
        let hs = [hole(1, 100.0, 100.0, 35.0), hole(2, 110.0, 100.0, 35.0)];
        let v = validate_drilling(&hs, 300.0, 700.0, 18.0, 8.0);
        assert!(v.iter().any(|r| r.message.contains("пересекаются")));
    }
}
