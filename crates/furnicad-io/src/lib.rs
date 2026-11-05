//! `furnicad-io` — import/export services (spec §15, §16).
//!
//! * project save/load: JSON bundle (`project.json`-style single file);
//! * mesh export: OBJ and STL (binary) from the display meshes;
//! * reports: cutting list (CSV) and drilling map (CSV).

pub mod obj;
pub mod project;
pub mod reports;
pub mod stl;

pub use obj::export_obj;
pub use project::{load_project, save_project, ProjectBundle};
pub use reports::{cutting_list_csv, drilling_map_csv};
pub use stl::export_stl_binary;

use furnicad_core::Result;
use furnicad_geom::Mesh3D;

/// Aggregate triangle count helper shared by exporters.
pub fn total_triangles(meshes: &[&Mesh3D]) -> usize {
    meshes.iter().map(|m| m.triangles.len()).sum()
}

/// Ensure parent directory exists before writing artifacts.
pub(crate) fn prepare_dir(path: &std::path::Path) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tri_count() {
        let mut m = Mesh3D::new();
        let v = furnicad_geom::mesh::Vertex { position: [0.0, 0.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] };
        m.push_tri(v, v, v);
        assert_eq!(total_triangles(&[&m]), 1);
    }
}
