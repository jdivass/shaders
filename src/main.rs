pub mod printer;
pub mod line;
pub mod framebuffer;
pub mod triangle;
pub mod transform;
pub mod object;
pub mod fragment;
pub mod vertex;
pub mod shaders;
pub mod primitive;
pub mod rasterizer;

use raylib::prelude::*;
use std::hint::black_box;
use std::time::Instant;

use crate::framebuffer::Framebuffer;
use crate::primitive::primitive_assembly;
use crate::rasterizer::rasterize;
use crate::shaders::{fragment_shader, vertex_shader};
use crate::triangle::TriangleMode;
use crate::vertex::Vertex;

pub struct Uniforms {
    pub model_matrix: Matrix,
}

fn main() {
    if std::env::args().any(|arg| {
        arg == "--benchmark" || arg == "--benchmark-triangles"
    }) {
        benchmark_rasterizers();
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(800, 600)
        .title("Primer triangulo")
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(800, 600);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));
    framebuffer.init_texture(&mut window, &thread);

    let positions = [
        Vector3::new(300.0, 200.0, 0.0),
        Vector3::new(400.0, 400.0, 0.0),
        Vector3::new(500.0, 200.0, 0.0),
    ];

    let vertex_array: Vec<Vertex> = positions
        .into_iter()
        .map(|position| {
            let mut vertex = Vertex::new_with_color(
                position,
                Vector3::new(1.0, 1.0, 1.0),
            );
            // Las posiciones ya están en coordenadas de pantalla.
            vertex.transformed_position = position;
            vertex
        })
        .collect();

    let uniforms = Uniforms {
        model_matrix: Matrix::identity(),
    };

    while !window.window_should_close() {
        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array);
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
    };
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, &uniforms))
        .collect();
    let triangles = primitive_assembly(&transformed_vertices);

    for mode in [TriangleMode::FilledBarycentric, TriangleMode::FilledScanline] {
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
) {
    // 1. Procesamiento de vértices: aplicar la misma matriz compartida a cada vértice.
    let processed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas: agrupar vértices procesados en triángulos.
    let triangles = primitive_assembly(&processed_vertices);

    // 3. Rasterización: reunir los fragmentos de todos los triángulos.
    let fragments = rasterize(&triangles, TriangleMode::FilledBarycentric);

    // 4. Procesamiento de fragmentos: escribirlos en el framebuffer.
    for fragment in fragments {
        let color = fragment_shader(&fragment);
        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            color,
        );
    }
}
