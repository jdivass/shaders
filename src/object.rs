use raylib::prelude::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<usize>,
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

fn parse_face_index(token: &str, vertex_count: usize) -> io::Result<usize> {
    let raw = token.split('/').next().unwrap_or("");
    let n: i64 = raw
        .parse()
        .map_err(|_| invalid(format!("Índice de cara inválido: '{}'", token)))?;

    let idx = if n > 0 {
        n - 1
    } else if n < 0 {
        vertex_count as i64 + n
    } else {
        return Err(invalid("Los índices en OBJ empiezan en 1, no en 0".to_string()));
    };

    if idx < 0 || idx as usize >= vertex_count {
        return Err(invalid(format!("Índice fuera de rango: '{}'", token)));
    }
    Ok(idx as usize)
}

pub fn load_obj(path: &str) -> io::Result<Obj> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vertices: Vec<Vector3> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let coords: Vec<f32> = parts
                    .take(3)
                    .map(|p| p.parse::<f32>())
                    .collect::<Result<_, _>>()
                    .map_err(|_| invalid(format!("Vértice inválido: '{}'", line)))?;

                if coords.len() < 3 {
                    return Err(invalid(format!("Vértice incompleto: '{}'", line)));
                }
                vertices.push(Vector3::new(coords[0], coords[1], coords[2]));
            }
            Some("f") => {
                let face: Vec<usize> = parts
                    .map(|t| parse_face_index(t, vertices.len()))
                    .collect::<io::Result<_>>()?;

                if face.len() < 3 {
                    return Err(invalid(format!("Cara con menos de 3 vértices: '{}'", line)));
                }

                for i in 1..face.len() - 1 {
                    indices.push(face[0]);
                    indices.push(face[i]);
                    indices.push(face[i + 1]);
                }
            }
            _ => {} // vn, vt, o, g, s, usemtl, etc. se ignoran
        }
    }

    Ok(Obj { vertices, indices })
}
