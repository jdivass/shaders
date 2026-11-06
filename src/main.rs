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
use crate::rasterizer::rasterize;
use crate::shaders::vertex_shader;
use crate::triangle::TriangleMode;
use crate::vertex::Vertex;

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub camera_position: Vector3,
    pub camera_target: Vector3,
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
        .title("3D triangle - orbit camera")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(800, 600);
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

    while !window.window_should_close() {
        let dt = window.get_frame_time();
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
        };
        framebuffer.clear();
        render(&mut framebuffer, &frame_uniforms, &vertex_array);
        framebuffer.swap_buffers(&mut window, &thread);
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

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
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
    let fragments = rasterize(&visible_triangles, TriangleMode::FilledBarycentric);

    // 4. Procesamiento de fragmentos: escribirlos en el framebuffer.
    let mut depth_buffer = vec![f32::INFINITY; (framebuffer.width * framebuffer.height) as usize];
    for fragment in fragments {
        let x = fragment.position.x as i32;
        let y = fragment.position.y as i32;
        if x < 0 || y < 0 || x as u32 >= framebuffer.width || y as u32 >= framebuffer.height {
            continue;
        }
        let index = y as usize * framebuffer.width as usize + x as usize;
        if fragment.depth >= depth_buffer[index] {
            continue;
        }
        depth_buffer[index] = fragment.depth;
        let color = planet::shade(&fragment, uniforms);
        framebuffer.point(x, y, color);
    }
}
