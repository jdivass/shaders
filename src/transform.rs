use raylib::prelude::*;
use std::f32::consts::PI;

pub fn transform (
    vertex: Vector3,
    translation: Vector3,
    scale: f32,
    center: Vector3,
    rotation: f32,
    ) -> Vector3 {
        let mut new_vertex = vertex;
        new_vertex -= center;
        let cos_theta = (rotation * PI/180.00).cos();
        let sin_theta = (rotation * PI/180.00).sin();
        let rotated_x =  new_vertex.x * cos_theta + new_vertex.y * sin_theta;
        let rotated_y = new_vertex.y * cos_theta + new_vertex.x * sin_theta;
        new_vertex.x = rotated_x;
        new_vertex.y = rotated_y;
        new_vertex += translation;
        new_vertex *= scale;
        new_vertex += center;
        new_vertex
        }
