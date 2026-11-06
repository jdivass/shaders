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

    let world = Vector3::new(transformed.x, transformed.y, transformed.z);
    let forward = (uniforms.camera_target - uniforms.camera_position).normalize();
    let right = forward.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
    let up = right.cross(forward).normalize();
    let relative = world - uniforms.camera_position;
    let depth = relative.dot(forward);
    let safe_depth = depth.max(0.001);
    let aspect = uniforms.screen_width / uniforms.screen_height;
    let focal = 1.0 / (60.0_f32.to_radians() * 0.5).tan();
    let ndc_x = relative.dot(right) * focal / (aspect * safe_depth);
    let ndc_y = relative.dot(up) * focal / safe_depth;

    let mut result = vertex.clone();
    result.transformed_position = Vector3::new(
        (ndc_x + 1.0) * uniforms.screen_width * 0.5,
        (1.0 - ndc_y) * uniforms.screen_height * 0.5,
        depth,
    );
    result
}

pub fn fragment_shader(fragment: &Fragment) -> Vector3 {
    fragment.color
}
