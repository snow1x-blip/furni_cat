//! The [`Document`] — root of a FurniCAD project.

use crate::entity::{EntityStore, MaterialLibrary, SelectionSet};
use crate::error::Result;
use crate::expr::{Expr, VariableTable};
use crate::ids::DocumentId;
use crate::units::Unit;
use serde::{Deserialize, Serialize};

/// Per-document settings (gaps, tolerances, machining defaults).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSettings {
    /// Default panel thickness in mm when none specified.
    pub default_thickness_mm: f64,
    /// Facade gap between neighboring fronts, mm.
    pub facade_gap_mm: f64,
    /// Edge band compensation applied when computing cut sizes.
    pub edge_compensation: bool,
    /// Minimum allowed hole-to-edge distance, mm (validation rule).
    pub min_hole_edge_distance: f64,
}

impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            default_thickness_mm: 18.0,
            facade_gap_mm: 3.0,
            edge_compensation: true,
            min_hole_edge_distance: 8.0,
        }
    }
}

/// A single named variable with an optional formula. When `formula` is present
/// its value is recomputed from the table on every rebuild; otherwise the
/// literal `value` is used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
}

/// Collection of document variables ("Переменные" node of the tree).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Variables {
    pub items: Vec<Variable>,
}

impl Variables {
    /// Define or overwrite a constant variable.
    pub fn set(&mut self, name: impl Into<String>, value: f64) {
        let name = name.into();
        if let Some(v) = self.items.iter_mut().find(|v| v.name == name) {
            v.value = value;
            v.formula = None;
        } else {
            self.items.push(Variable {
                name,
                value,
                formula: None,
            });
        }
    }

    /// Define a formula-driven variable; the current value becomes stale until
    /// the next [`Variables::recompute`].
    pub fn set_formula(&mut self, name: impl Into<String>, formula: impl Into<String>) -> Result<()> {
        // Validate parseability eagerly so bad formulas are rejected at edit time.
        let formula = formula.into();
        Expr::parse_check(&formula)?;
        let name = name.into();
        if let Some(v) = self.items.iter_mut().find(|v| v.name == name) {
            v.formula = Some(formula);
        } else {
            self.items.push(Variable {
                name,
                value: 0.0,
                formula: Some(formula),
            });
        }
        Ok(())
    }

    /// Flatten into an evaluation table. Formula variables are evaluated in
    /// definition order — forward references resolve if defined earlier.
    pub fn to_table(&self) -> Result<VariableTable> {
        let mut table = VariableTable::new();
        for v in &self.items {
            let val = match &v.formula {
                Some(f) => crate::expr::parse_expr(f)?.eval(&table)?,
                None => v.value,
            };
            table.set(v.name.clone(), val);
        }
        Ok(table)
    }

    pub fn get(&self, name: &str) -> Option<&Variable> {
        self.items.iter().find(|v| v.name == name)
    }
}

/// Root document object holding all model data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub name: String,
    pub units: Unit,
    pub variables: Variables,
    pub materials: MaterialLibrary,
    pub entities: EntityStore,
    pub selection: SelectionSet,
    pub settings: DocumentSettings,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            id: DocumentId(1),
            name: "Без имени".into(),
            units: Unit::Millimeter,
            variables: Variables::default(),
            materials: MaterialLibrary::new(),
            entities: EntityStore::new(),
            selection: SelectionSet::default(),
            settings: DocumentSettings::default(),
        }
    }
}

impl Document {
    /// Create an empty document with standard furniture variables.
    pub fn new(name: impl Into<String>) -> Self {
        let mut doc = Self::default();
        doc.name = name.into();
        doc.variables.set("width", 600.0);
        doc.variables.set("height", 720.0);
        doc.variables.set("depth", 560.0);
        doc.variables.set("panel_thickness", 18.0);
        doc.variables.set("gap", 3.0);
        doc
    }

    /// Evaluate all variables into a lookup table.
    pub fn variable_table(&self) -> Result<VariableTable> {
        self.variables.to_table()
    }

    /// Resolve a possibly parametric length: constants pass through, variables
    /// are looked up in the table.
    pub fn eval_length(&self, expr: &Expr) -> Result<f64> {
        expr.eval(&self.variable_table()?)
    }
}

/// Helper trait so [`Variables::set_formula`] can validate syntax cheaply.
trait ParseCheck: Sized {
    fn parse_check(text: &str) -> Result<Self>;
}

impl ParseCheck for Expr {
    fn parse_check(text: &str) -> Result<Self> {
        crate::expr::parse_expr(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_defaults_have_variables() {
        let doc = Document::new("Шкаф");
        let t = doc.variable_table().unwrap();
        assert_eq!(t.get("width"), Some(600.0));
        assert_eq!(t.get("panel_thickness"), Some(18.0));
    }

    #[test]
    fn formula_variable_recomputes() {
        let mut vars = Variables::default();
        vars.set("width", 600.0);
        vars.set("gap", 3.0);
        vars.set_formula("facade_width", "(width - 2 * gap) / 2")
            .unwrap();
        let t = vars.to_table().unwrap();
        assert!((t.get("facade_width").unwrap() - 297.0).abs() < 1e-9);
    }

    #[test]
    fn bad_formula_rejected() {
        let mut vars = Variables::default();
        assert!(vars.set_formula("x", "1 +").is_err());
    }

    #[test]
    fn eval_length_from_doc() {
        let doc = Document::new("Тест");
        let e = crate::expr::parse_expr("width + panel_thickness").unwrap();
        assert!((doc.eval_length(&e).unwrap() - 618.0).abs() < 1e-9);
    }
}
