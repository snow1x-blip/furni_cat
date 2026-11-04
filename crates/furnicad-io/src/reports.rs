//! Production reports: cutting list and drilling map (spec §15).

use furnicad_core::Result;
use furnicad_panels::panel::{Hole, Panel};

/// CSV cutting list: name, material, size W×H×T, quantity of identical parts.
pub fn cutting_list_csv(panels: &[(Panel, f64, String)]) -> Result<String> {
    let mut s = String::new();
    let mut w = csv_writer(&mut s);
    w.write_row(&["Деталь", "Материал", "Длина", "Ширина", "Толщина", "Кол-во"])?;
    // group identical sizes
    let mut groups: Vec<(String, String, f64, f64, f64, usize)> = Vec::new();
    for (p, t, mat) in panels {
        let (w_, h_) = p.size();
        let key = (mat.clone(), round(w_), round(h_), round(*t));
        if let Some(g) = groups
            .iter_mut()
            .find(|g| (g.1.clone(), round(g.2), round(g.3), round(g.4)) == key)
        {
            g.5 += 1;
        } else {
            groups.push((p.name.clone(), mat.clone(), w_, h_, *t, 1));
        }
    }
    for (name, mat, l, wdt, th, qty) in groups {
        w.write_row(&[
            name.as_str(),
            mat.as_str(),
            &fmt(l),
            &fmt(wdt),
            &fmt(th),
            &qty.to_string(),
        ])?;
    }
    Ok(s)
}

/// CSV drilling map: panel, hole id, X, Y, Z-face, diameter, depth, type.
pub fn drilling_map_csv(panel_name: &str, thickness: f64, holes: &[Hole]) -> Result<String> {
    let mut s = String::new();
    let mut w = csv_writer(&mut s);
    w.write_row(&["Панель", "Отв.", "X", "Y", "Сторона", "Ø", "Глубина", "Тип"])?;
    for h in holes {
        let depth = match h.depth {
            furnicad_panels::panel::HoleDepth::Through => "сквозь".to_string(),
            furnicad_panels::panel::HoleDepth::Blind(d) => fmt(d),
        };
        let dia = match &h.diameter {
            furnicad_core::Expr::Const(v) => fmt(*v),
            other => format!("{other:?}"),
        };
        w.write_row(&[
            panel_name,
            &h.id.raw().to_string(),
            &fmt(h.position[0]),
            &fmt(h.position[1]),
            if h.from_front { "лицо" } else { "тыл" },
            &dia,
            &depth,
            &format!("{:?}", h.hole_type),
        ])?;
    }
    let _ = thickness;
    Ok(s)
}

struct CsvWriter<'a> {
    out: &'a mut String,
}
impl<'a> CsvWriter<'a> {
    fn write_row(&mut self, cells: &[&str]) -> Result<()> {
        let esc = |c: &str| c.replace('"', "\"\"");
        self.out.push_str(
            &cells
                .iter()
                .map(|c| format!("\"{}\"", esc(c)))
                .collect::<Vec<_>>()
                .join(";"),
        );
        self.out.push('\n');
        Ok(())
    }
}
fn csv_writer(s: &mut String) -> CsvWriter<'_> {
    CsvWriter { out: s }
}

fn fmt(v: f64) -> String {
    format!("{:.1}", v)
}
fn round(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use furnicad_core::{MaterialId, PanelId};
    use furnicad_panels::panel::PanelRole;

    #[test]
    fn groups_identical_panels() {
        let a = Panel::rect(PanelId(1), "Боковина", PanelRole::Side, MaterialId(1), 300.0, 600.0, 18.0, [0.0; 3]);
        let b = Panel::rect(PanelId(2), "Боковина", PanelRole::Side, MaterialId(1), 300.0, 600.0, 18.0, [0.0; 3]);
        let csv = cutting_list_csv(&[(a, 18.0, "ЛДСП Дуб".into()), (b, 18.0, "ЛДСП Дуб".into())]).unwrap();
        assert!(csv.contains("\"2\""), "{csv}");
    }
}
