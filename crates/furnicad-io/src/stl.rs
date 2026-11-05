//! Binary STL exporter for 3D printing / CNC reference models.

use furnicad_core::Result;
use furnicad_geom::Mesh3D;
use std::io::Write;

/// Export meshes as a binary STL (80-byte header, u32 count, per-tri records).
pub fn export_stl_binary(path: &str, meshes: &[&Mesh3D]) -> Result<()> {
    crate::prepare_dir(std::path::Path::new(path))?;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut header = [0u8; 80];
    let tag = b"FurniCAD STL";
    header[..tag.len()].copy_from_slice(tag);
    f.write_all(&header)?;
    let count: u32 = meshes.iter().map(|m| m.triangles.len() as u32).sum();
    f.write_all(&count.to_le_bytes())?;
    for m in meshes {
        for t in &m.triangles {
            let p = |i: u32| m.vertices[i as usize].position.map(|x| x as f32);
            let (a, b, c) = (p(t[0]), p(t[1]), p(t[2]));
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
            let nn = [n[0] / l, n[1] / l, n[2] / l];
            f.write_all(&nn.map(f32::to_le_bytes).concat())?;
            for vtx in [a, b, c] {
                f.write_all(&vtx.map(f32::to_le_bytes).concat())?;
            }
            f.write_all(&0u16.to_le_bytes())?;
        }
    }
    f.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_header_and_count() {
        let mut m = Mesh3D::new();
        let mk = |p: [f64; 3]| furnicad_geom::mesh::Vertex { position: p, normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] };
        m.push_tri(mk([0.0, 0.0, 0.0]), mk([1.0, 0.0, 0.0]), mk([0.0, 1.0, 0.0]));
        let path = std::env::temp_dir().join("furnicad_test.stl");
        export_stl_binary(path.to_str().unwrap(), &[&m]).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..12], b"FurniCAD STL");
        assert_eq!(u32::from_le_bytes(bytes[80..84].try_into().unwrap()), 1);
        assert_eq!(bytes.len(), 84 + 50);
        std::fs::remove_file(path).ok();
    }
}
