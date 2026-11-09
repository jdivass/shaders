pub mod fragment;
pub mod framebuffer;
pub mod line;
pub mod object;
pub mod planet;
pub mod primitive;
pub mod printer;
pub mod rasterizer;
pub mod shaders;
pub mod transform;
pub mod triangle;
pub mod vertex;

use raylib::prelude::*;
use std::hint::black_box;
use std::time::Instant;

use crate::framebuffer::Framebuffer;
use crate::primitive::primitive_assembly;
use crate::rasterizer::{rasterize, rasterize_each};
use crate::shaders::vertex_shader;
use crate::triangle::TriangleMode;
use crate::vertex::Vertex;

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub camera_position: Vector3,
    pub camera_target: Vector3,
    pub screen_width: f32,
    pub screen_height: f32,
    pub time: f32,
    pub cloud_coverage: f32,
    pub planet_kind: PlanetKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlanetKind {
    Rocky,
    GasGiant,
    Star,
}

struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    target: Vector3,
}

impl OrbitCamera {
    fn position(&self) -> Vector3 {
        let horizontal = self.distance * self.pitch.cos();
        Vector3::new(
            self.target.x + horizontal * self.yaw.sin(),
            self.target.y + self.distance * self.pitch.sin(),
            self.target.z + horizontal * self.yaw.cos(),
        )
    }
}

fn main() {
    if std::env::args().any(|arg| arg == "--benchmark" || arg == "--benchmark-triangles") {
        benchmark_rasterizers();
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(800, 600)
        .title("Planeta rocoso")
        .build();

    let monitor = raylib::core::window::get_current_monitor();
    let monitor_width = raylib::core::window::get_monitor_width(monitor);
    let monitor_height = raylib::core::window::get_monitor_height(monitor);
    let window_width = (monitor_width as f32 * 0.7).round() as i32;
    let window_height = (monitor_height as f32 * 0.7).round() as i32;
    window.set_window_size(window_width, window_height);
    window.set_window_position(
        (monitor_width - window_width) / 2,
        (monitor_height - window_height) / 2,
    );
    window.set_target_fps(60);

    let display_width = window.get_screen_width();
    let display_height = window.get_screen_height();
    // Limitar el trabajo del rasterizador en pantallas de alta resolución.
    let scale = (1280.0 / display_width as f32)
        .min(720.0 / display_height as f32)
        .min(1.0);
    let render_width = (display_width as f32 * scale).round().max(1.0) as u32;
    let render_height = (display_height as f32 * scale).round().max(1.0) as u32;
    let mut framebuffer = Framebuffer::new(render_width, render_height);
    let mut depth_buffer = vec![f32::INFINITY; (render_width * render_height) as usize];
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    let mesh = object::load_obj("model/sphere.obj").expect("No se pudo cargar model/sphere.obj");
    let vertex_array: Vec<Vertex> = mesh
        .indices
        .iter()
        .map(|&index| Vertex::new_with_color(mesh.vertices[index], Vector3::new(1.0, 1.0, 1.0)))
        .collect();
    let mut camera = OrbitCamera {
        yaw: 0.0,
        pitch: 0.0,
        distance: 3.0,
        target: Vector3::zero(),
    };
    let start_time = Instant::now();
    let mut cloud_coverage = 0.55_f32;
    let mut planet_kind = PlanetKind::Rocky;

    while !window.window_should_close() {
        let dt = window.get_frame_time();
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            planet_kind = PlanetKind::Rocky;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            planet_kind = PlanetKind::GasGiant;
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            planet_kind = PlanetKind::Star;
        }
        if window.is_key_down(KeyboardKey::KEY_PAGE_UP) {
            cloud_coverage = (cloud_coverage + dt * 0.35).clamp(0.0, 1.0);
        }
        if window.is_key_down(KeyboardKey::KEY_PAGE_DOWN) {
            cloud_coverage = (cloud_coverage - dt * 0.35).clamp(0.0, 1.0);
        }
        let orbit_speed = 1.8 * dt;
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.yaw -= orbit_speed;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.yaw += orbit_speed;
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.pitch = (camera.pitch + orbit_speed).clamp(-1.45, 1.45);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.pitch = (camera.pitch - orbit_speed).clamp(-1.45, 1.45);
        }
        if window.is_key_down(KeyboardKey::KEY_W) {
            camera.distance = (camera.distance - 3.0 * dt).max(1.5);
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            camera.distance += 3.0 * dt;
        }
        if window.is_key_down(KeyboardKey::KEY_A) {
            camera.target.x -= 2.0 * dt;
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            camera.target.x += 2.0 * dt;
        }
        let frame_uniforms = Uniforms {
            model_matrix: Matrix::identity(),
            camera_position: camera.position(),
            camera_target: camera.target,
            screen_width: render_width as f32,
            screen_height: render_height as f32,
            time: start_time.elapsed().as_secs_f32(),
            cloud_coverage,
            planet_kind,
        };
        framebuffer.clear();
        render(
            &mut framebuffer,
            &frame_uniforms,
            &vertex_array,
            &mut depth_buffer,
        );
        framebuffer.swap_buffers(&mut window, &thread, display_width, display_height);
    }
}

fn benchmark_rasterizers() {
    const RUNS: usize = 7;
    const ITERATIONS_PER_RUN: usize = 100;

    let positions = [
        Vector3::new(100.0, 80.0, 0.0),
        Vector3::new(400.0, 520.0, 0.0),
        Vector3::new(700.0, 100.0, 0.0),
    ];
    let vertex_array: Vec<Vertex> = positions
        .into_iter()
        .map(|position| Vertex::new_with_color(position, Vector3::new(1.0, 1.0, 1.0)))
        .collect();
    let uniforms = Uniforms {
        model_matrix: Matrix::identity(),
        camera_position: Vector3::new(0.0, 0.0, 5.0),
        camera_target: Vector3::zero(),
        screen_width: 800.0,
        screen_height: 600.0,
        time: 0.0,
        cloud_coverage: 0.55,
        planet_kind: PlanetKind::Rocky,
    };
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, &uniforms))
        .collect();
    let triangles = primitive_assembly(&transformed_vertices);

    for mode in [
        TriangleMode::FilledBarycentric,
        TriangleMode::FilledScanline,
    ] {
        // Warm up once, outside the measured batches.
        black_box(rasterize(black_box(&triangles), mode));

        let mut samples = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let start = Instant::now();
            for _ in 0..ITERATIONS_PER_RUN {
                let fragments = rasterize(black_box(&triangles), mode);
                black_box(fragments);
            }
            samples.push(start.elapsed());
        }

        samples.sort_unstable();
        let median = samples[RUNS / 2];
        let fragment_count = rasterize(&triangles, mode).len();
        let name = match mode {
            TriangleMode::FilledBarycentric => "baricéntrico",
            TriangleMode::FilledScanline => "scanline",
            TriangleMode::Wireframe => unreachable!(),
        };
        println!(
            "{name}: mediana de {RUNS} ejecuciones × {ITERATIONS_PER_RUN}: {median:?}; {fragment_count} fragmentos"
        );
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    depth_buffer: &mut [f32],
) {
    depth_buffer.fill(f32::INFINITY);
    // 1. Procesamiento de vértices: aplicar la misma matriz compartida a cada vértice.
    let processed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas: agrupar vértices procesados en triángulos.
    let triangles = primitive_assembly(&processed_vertices);

    // 3. Rasterización: reunir los fragmentos de todos los triángulos.
    let visible_triangles: Vec<[Vertex; 3]> = triangles
        .into_iter()
        .filter(|triangle| {
            triangle
                .iter()
                .all(|vertex| vertex.transformed_position.z > 0.0)
        })
        .collect();
    // 4. Procesar fragmentos conforme se rasterizan para evitar un Vec enorme por cuadro.
    rasterize_each(
        &visible_triangles,
        TriangleMode::FilledScanline,
        |fragment| {
            let x = fragment.position.x as i32;
            let y = fragment.position.y as i32;
            if x < 0
                || y < 0
                || x as u32 >= framebuffer.width
                || y as u32 >= framebuffer.height
            {
                return;
            }
            let index = y as usize * framebuffer.width as usize + x as usize;
            if fragment.depth >= depth_buffer[index] {
                return;
            }
            depth_buffer[index] = fragment.depth;
            let color = planet::shade(&fragment, uniforms);
            framebuffer.point(x, y, color);
        },
    );
}
