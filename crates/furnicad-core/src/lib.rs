//! `furnicad-core` — foundational types of the FurniCAD parametric furniture
//! CAD system.
//!
//! This crate contains no geometry algorithms and no UI: only the stable data
//! model shared by every other layer:
//!
//! * [`ids`] — strongly typed entity identifiers;
//! * [`units`] — mm-based unit handling and tolerances;
//! * [`expr`] — parametric expressions (`Expr`) with a formula parser;
//! * [`error`] — [`CadError`]/[`Result`] used across the workspace;
//! * [`entity`] — generic entity store, materials, selection;
//! * [`document`] — the root [`Document`] object;
//! * [`transform`] — rigid transforms built on `glam`.

pub mod document;
pub mod entity;
pub mod error;
pub mod expr;
pub mod ids;
pub mod transform;
pub mod units;

pub use document::{Document, DocumentSettings, Variable, Variables};
pub use entity::{EntityPayload, EntityRecord, EntityStore, Material, MaterialKind, MaterialLibrary, Metadata, SelectionSet};
pub use error::{CadError, Result};
pub use expr::{parse_expr, Expr, VariableTable};
pub use ids::*;
pub use transform::Transform;
pub use units::{Tolerance, Unit};

/// Crate version string baked at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_reexports_work() {
        let doc = Document::new("Полка");
        assert_eq!(doc.units, Unit::Millimeter);
        let id: EntityId = EntityId(1);
        assert_eq!(id.raw(), 1);
    }
}
