use raylib::prelude::*;

use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::Uniforms;

fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x
            + matrix.m4 * vector.y
            + matrix.m8 * vector.z
            + matrix.m12 * vector.w,
        matrix.m1 * vector.x
            + matrix.m5 * vector.y
            + matrix.m9 * vector.z
            + matrix.m13 * vector.w,
        matrix.m2 * vector.x
            + matrix.m6 * vector.y
            + matrix.m10 * vector.z
            + matrix.m14 * vector.w,
        matrix.m3 * vector.x
            + matrix.m7 * vector.y
            + matrix.m11 * vector.z
            + matrix.m15 * vector.w,
    )
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vector4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0,
    );

    let transformed = multiply_matrix_vector4(&uniforms.model_matrix, &position);

    let mut result = vertex.clone();
    result.transformed_position = Vector3::new(
        transformed.x,
        transformed.y,
        transformed.z,
    );
    result
}

pub fn fragment_shader(fragment: &Fragment) -> Vector3 {
    fragment.color
}
