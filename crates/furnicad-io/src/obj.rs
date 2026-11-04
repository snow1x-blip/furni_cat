//! Wavefront OBJ exporter.

use furnicad_core::Result;
use furnicad_geom::Mesh3D;
use std::io::Write;

/// Write several named meshes into one .obj file.
pub fn export_obj(path: &str, objects: &[(&str, &Mesh3D)]) -> Result<()> {
    crate::prepare_dir(std::path::Path::new(path))?;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(f, "# FurniCAD export")?;
    let mut base = 0usize;
    for (name, m) in objects {
        writeln!(f, "o {name}")?;
        for v in &m.vertices {
            writeln!(f, "v {:.4} {:.4} {:.4}", v.position[0], v.position[1], v.position[2])?;
        }
        for v in &m.vertices {
            writeln!(f, "vn {:.4} {:.4} {:.4}", v.normal[0], v.normal[1], v.normal[2])?;
        }
        for t in &m.triangles {
            writeln!(
                f,
                "f {}//{} {}//{} {}//{}",
                t[0] + 1 + base,
                t[0] + 1 + base,
                t[1] + 1 + base,
                t[1] + 1 + base,
                t[2] + 1 + base,
                t[2] + 1 + base
            )?;
        }
        base += m.vertices.len();
    }
    f.flush()?;
    Ok(())
}

/// Parse a minimal subset of OBJ (v / f with optional //vn) back into a mesh —
/// enough for decorative models import (spec §15).
pub fn import_obj(path: &str) -> Result<Mesh3D> {
    use furnicad_geom::mesh::Vertex;
    let text = std::fs::read_to_string(path)?;
    let mut positions: Vec<[f64; 3]> = Vec::new();
    let mut mesh = Mesh3D::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => {
                let p: Vec<f64> = it.filter_map(|s| s.parse().ok()).collect();
                if p.len() >= 3 {
                    positions.push([p[0], p[1], p[2]]);
                }
            }
            Some("f") => {
                let idx: Vec<usize> = it
                    .filter_map(|s| s.split('/').next().and_then(|t| t.parse::<usize>().ok()))
                    .map(|i| i.saturating_sub(1))
                    .collect();
                // fan triangulation
                if idx.len() >= 3 {
                    for w in idx[1..].windows(2) {
                        let tri = [idx[0], w[0], w[1]];
                        let vs = tri.map(|i| positions.get(i).copied().unwrap_or([0.0; 3]));
                        let n = [0.0, 0.0, 1.0];
                        mesh.push_tri(
                            Vertex { position: vs[0], normal: n, uv: [0.0, 0.0] },
                            Vertex { position: vs[1], normal: n, uv: [0.0, 0.0] },
                            Vertex { position: vs[2], normal: n, uv: [0.0, 0.0] },
                        );
                    }
                }
            }
            _ => {}
        }
    }
    mesh.compute_flat_normals();
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_triangle() {
        let mut m = Mesh3D::new();
        let mk = |p: [f64; 3]| furnicad_geom::mesh::Vertex { position: p, normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] };
        m.push_tri(mk([0.0, 0.0, 0.0]), mk([1.0, 0.0, 0.0]), mk([0.0, 1.0, 0.0]));
        let path = std::env::temp_dir().join("furnicad_test.obj");
        export_obj(path.to_str().unwrap(), &[("tri", &m)]).unwrap();
        let back = import_obj(path.to_str().unwrap()).unwrap();
        assert_eq!(back.triangles.len(), 1);
        std::fs::remove_file(path).ok();
    }
}
