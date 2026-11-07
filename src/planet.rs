use crate::{PlanetKind, Uniforms, fragment::Fragment};
use raylib::prelude::*;

fn mix(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a * (1.0 - t) + b * t
}
fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn hash(p: Vector3) -> f32 {
    let x = p.x as i32 as u32;
    let y = p.y as i32 as u32;
    let z = p.z as i32 as u32;
    let mut value = x.wrapping_mul(0x8da6_b343)
        ^ y.wrapping_mul(0xd816_3841)
        ^ z.wrapping_mul(0xcb1a_b31f);
    value ^= value >> 13;
    value = value.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    (value as f32 / u32::MAX as f32) * 2.0 - 1.0
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

fn warp_into_vortex(position: Vector3, center: Vector3) -> Vector3 {
    let axis = center.normalize();
    let radial_component = position.dot(axis);
    let tangent = position - axis * radial_component;
    let radius = tangent.dot(tangent).sqrt();
    let influence = 1.0 - smoothstep(0.0, 0.40, radius);
    let angle = influence * 10.05;
    let (sine, cosine) = angle.sin_cos();

    (position * cosine)
        + (axis.cross(position) * sine)
        + (axis * (radial_component * (1.0 - cosine)))
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let n = fragment.normal.normalize();
    // Luz direccional fija: el terminador divide claramente ambos hemisferios.
    let sun_direction = Vector3::new(-0.55, 0.32, 0.77).normalize();
    let sun_facing = n.dot(sun_direction);
    let diffuse = sun_facing.max(0.0);
    let night = 1.0 - smoothstep(-0.12, 0.12, sun_facing);
    let view = (uniforms.camera_position - p).normalize();
    if uniforms.planet_kind == PlanetKind::Star {
        return shade_star(p, n, view, uniforms);
    }
    if uniforms.planet_kind == PlanetKind::GasGiant {
        return shade_gas_giant(p, n, diffuse, view, uniforms);
    }
    let half_vector = (sun_direction + view).normalize();
    let highlight = n.dot(half_vector).max(0.0);
    // Potencia 32 con multiplicaciones para evitar un powf por fragmento.
    let highlight2 = highlight * highlight;
    let highlight4 = highlight2 * highlight2;
    let highlight8 = highlight4 * highlight4;
    let highlight16 = highlight8 * highlight8;
    let specular = highlight16 * highlight16;
    let rim_factor = 1.0 - n.dot(view).max(0.0);
    let rim = rim_factor * rim_factor * rim_factor;
    let height = fbm(p * 4.0);
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

    // Luces agrupadas en tierra firme y visibles únicamente en el hemisferio nocturno.
    let land_mask = smoothstep(0.46, 0.54, height);
    let ocean_mask = 1.0 - land_mask;
    let city_regions = smoothstep(0.53, 0.72, fbm(p * 18.0 + Vector3::new(7.0, 2.0, 11.0)));
    let city_lights = smoothstep(0.60, 0.78, noise(p * 72.0 + Vector3::new(13.0, 5.0, 3.0)));
    let lights = land_mask * city_regions * city_lights * night;

    let surface_color = base * (0.035 + diffuse * 0.965)
        + Vector3::new(0.82, 0.9, 1.0) * (specular * ocean_mask * diffuse * 0.9)
        + Vector3::new(1.0, 0.48, 0.12) * (lights * 1.6)
        + Vector3::new(0.08, 0.16, 0.3) * (rim * 0.35);

    // Las nubes usan un desplazamiento propio y se desplazan sobre la superficie fija.
    let cloud_drift = Vector3::new(uniforms.time * 0.025, 0.0, uniforms.time * 0.012);
    let cloud_field = (noise(p * 6.0 + cloud_drift) * 0.65
        + noise(p * 12.0 + cloud_drift * 1.7) * 0.35)
        * 0.5
        + 0.5;
    let cloud_edge = 0.95 - uniforms.cloud_coverage * 0.8;
    let cloud_mask = smoothstep(cloud_edge, cloud_edge + 0.14, cloud_field)
        * uniforms.cloud_coverage;
    let cloud_color = Vector3::new(0.82, 0.87, 0.94) * (0.18 + diffuse * 0.82);
    let cloud_opacity = cloud_mask * 0.78;

    mix(surface_color, cloud_color, cloud_opacity)
}

fn shade_star(p: Vector3, n: Vector3, view: Vector3, uniforms: &Uniforms) -> Vector3 {
    let time = uniforms.time;
    let drift = Vector3::new(time * 0.03, time * 0.02, -time * 0.025);
    let plasma = fbm(p * 5.0 + drift);
    let warp = (p.x * 11.0 + time).sin() * (p.z * 13.0 - time * 0.7).sin();
    let wave = (p.y * 24.0 + warp * 3.0 + time * 1.5).sin() * 0.5 + 0.5;
    let cells = (p.x * 31.0 + p.z * 17.0 + time).sin() * 0.5 + 0.5;
    let heat = (wave * 0.65 + cells * 0.2 + plasma * 0.15).clamp(0.0, 1.0);
    let heat = heat * heat.sqrt();
    let color = mix(
        Vector3::new(0.42, 0.012, 0.001),
        Vector3::new(1.0, 0.82, 0.24),
        heat,
    );
    let eruptions = smoothstep(0.58, 0.9, noise(p * 16.0 + drift));
    let facing = n.dot(view).max(0.0);
    let limb = 1.0 - facing;
    let limb = limb * limb;

    color * (0.58 + facing * 0.42)
        + Vector3::new(1.0, 0.25, 0.025) * (eruptions * 0.24)
        + Vector3::new(1.0, 0.28, 0.035) * (limb * 0.22)
}

fn shade_gas_giant(
    p: Vector3,
    n: Vector3,
    diffuse: f32,
    view: Vector3,
    uniforms: &Uniforms,
) -> Vector3 {
    let drift = Vector3::new(uniforms.time * 0.035, 0.0, uniforms.time * 0.012);
    let storm_center = Vector3::new(0.24, 0.17, 0.96).normalize();
    let warped_normal = warp_into_vortex(n, storm_center);
    let warped_position = warped_normal * p.dot(n).max(0.0);

    // La deformación afecta las bandas y su ruido, no solo el color de la tormenta.
    let turbulence = noise(warped_position * 6.0 + drift);
    let wave = (warped_position.y * 30.0 + turbulence * 3.0 + uniforms.time * 0.035).sin() * 0.5 + 0.5;
    let band = smoothstep(0.34, 0.66, wave);
    let mut color = mix(
        Vector3::new(0.003, 0.12, 0.35),
        Vector3::new(0.2, 0.55, 0.9),
        band,
    );

    let fine_wave = (warped_position.y * 78.0 + noise(warped_position * 11.0 + drift) * 2.0).sin() * 1.0 + 1.0;
    color = mix(color, Vector3::new(1.0, 1.0, 1.0), smoothstep(0.91, 0.98, fine_wave) * 0.45);

    // Una tormenta ovalada permanece en coordenadas del planeta mientras las bandas fluyen.
    let storm_distance = 1.0 - n.dot(storm_center).clamp(-1.0, 1.0);
    let storm = 1.0 - smoothstep(0.025, 0.075, storm_distance);
    let storm_color = Vector3::new(0.369, 0.369, 0.388);
    color = mix(color, storm_color, storm * 0.9);

    let sun_direction = Vector3::new(-0.55, 0.32, 0.77).normalize();
    let half_vector = (sun_direction + view).normalize();
    let highlight = n.dot(half_vector).max(0.0);
    let highlight2 = highlight * highlight;
    let highlight4 = highlight2 * highlight2;
    let highlight8 = highlight4 * highlight4;
    let highlight16 = highlight8 * highlight8;
    let specular = highlight16 * highlight16;
    let rim_factor = 1.0 - n.dot(view).max(0.0);
    let rim = rim_factor * rim_factor * rim_factor;

    color * (0.035 + diffuse * 0.965)
        + Vector3::new(1.0, 0.78, 0.5) * (specular * diffuse * 0.35)
        + Vector3::new(0.12, 0.2, 0.34) * (rim * 0.35)
}
