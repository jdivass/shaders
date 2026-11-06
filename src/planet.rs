use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

fn mix(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a * (1.0 - t) + b * t
}
fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn hash(p: Vector3) -> f32 {
    let n = (p.dot(Vector3::new(127.1, 311.7, 74.7))).sin() * 43758.5453;
    (n - n.floor()) * 2.0 - 1.0
}
fn noise(p: Vector3) -> f32 {
    let i = Vector3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = Vector3::new(p.x.fract(), p.y.fract(), p.z.fract());
    let u = Vector3::new(
        f.x * f.x * (3.0 - 2.0 * f.x),
        f.y * f.y * (3.0 - 2.0 * f.y),
        f.z * f.z * (3.0 - 2.0 * f.z),
    );
    let h = |x: f32, y: f32, z: f32| hash(i + Vector3::new(x, y, z));
    let lerp = |a: f32, b: f32, t: f32| a * (1.0 - t) + b * t;
    let x00 = lerp(h(0.0, 0.0, 0.0), h(1.0, 0.0, 0.0), u.x);
    let x10 = lerp(h(0.0, 1.0, 0.0), h(1.0, 1.0, 0.0), u.x);
    let x01 = lerp(h(0.0, 0.0, 1.0), h(1.0, 0.0, 1.0), u.x);
    let x11 = lerp(h(0.0, 1.0, 1.0), h(1.0, 1.0, 1.0), u.x);
    lerp(lerp(x00, x10, u.y), lerp(x01, x11, u.y), u.z)
}
fn fbm(mut p: Vector3) -> f32 {
    let mut value = 0.0;
    let mut amp = 0.5;
    let mut total = 0.0;
    for _ in 0..4 {
        value += noise(p) * amp;
        total += amp;
        p *= 2.0;
        amp *= 0.5;
    }
    value / total * 0.5 + 0.5
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let n = fragment.normal.normalize();
    let light_dir = (uniforms.camera_position - p).normalize();
    let diffuse = n.dot(light_dir).max(0.0);
    let view = (uniforms.camera_position - p).normalize();
    let rim = (1.0 - n.dot(view).max(0.0)).powf(3.0);
    let light = 0.12 + diffuse * 0.88;
    let height = fbm(p * 3.0);
    let ocean = Vector3::new(0.02, 0.1, 0.35);
    let land = mix(
        Vector3::new(0.12, 0.3, 0.1),
        Vector3::new(0.65, 0.52, 0.3),
        height,
    );
    let color = mix(ocean, land, smoothstep(0.48, 0.52, height));
    let base = mix(
        color,
        Vector3::new(0.9, 0.95, 1.0),
        smoothstep(0.65, 0.78, height),
    );
    base * light + Vector3::new(0.08, 0.16, 0.3) * rim
}
