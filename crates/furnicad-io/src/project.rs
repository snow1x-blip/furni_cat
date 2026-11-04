//! Project bundle save/load — the internal format of spec §16.
//!
//! The whole parametric state (document + history graph) is serialized into a
//! single `.furnicad` JSON file. Geometry caches are rebuilt on load because
//! operations are the source of truth.

use furnicad_core::{Document, Result};
use furnicad_history::HistoryGraph;
use serde::{Deserialize, Serialize};

/// On-disk project container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBundle {
    pub format_version: u32,
    pub document: Document,
    pub history: HistoryGraph,
}

impl ProjectBundle {
    pub fn new(document: Document, history: HistoryGraph) -> Self {
        Self { format_version: 1, document, history }
    }
}

/// Save the project to `path` (pretty JSON).
pub fn save_project(path: &str, bundle: &ProjectBundle) -> Result<()> {
    crate::prepare_dir(std::path::Path::new(path))?;
    let json = serde_json::to_string_pretty(bundle)
        .map_err(|e| furnicad_core::CadError::Serialization(e.to_string()))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load a previously saved project.
pub fn load_project(path: &str) -> Result<ProjectBundle> {
    let json = std::fs::read_to_string(path)?;
    serde_json::from_str(&json).map_err(|e| furnicad_core::CadError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_roundtrip() {
        let doc = Document::new("Шкаф");
        let hist = HistoryGraph::new();
        let bundle = ProjectBundle::new(doc, hist);
        let path = std::env::temp_dir().join("furnicad_proj.furnicad");
        save_project(path.to_str().unwrap(), &bundle).unwrap();
        let back = load_project(path.to_str().unwrap()).unwrap();
        assert_eq!(back.document.name, "Шкаф");
        assert_eq!(back.format_version, 1);
        std::fs::remove_file(path).ok();
    }
}
