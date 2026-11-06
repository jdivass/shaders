use crate::fragment::Fragment;
use crate::vertex::Vertex;
use raylib::math::Vector3;

#[derive(Clone, Copy)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
    FilledScanline,
}

fn append_edge(fragments: &mut Vec<Fragment>, start: &Vertex, end: &Vertex) {
    let mut x = start.transformed_position.x as i32;
    let mut y = start.transformed_position.y as i32;
    let x1 = end.transformed_position.x as i32;
    let y1 = end.transformed_position.y as i32;
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        fragments.push(Fragment::new(x as f32, y as f32, start.color, 0.0));
        if x == x1 && y == y1 {
            break;
        }
        let twice_error = 2 * error;
        if twice_error >= dy {
            error += dy;
            x += sx;
        }
        if twice_error <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn wireframe_triangle(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    append_edge(&mut fragments, a, b);
    append_edge(&mut fragments, b, c);
    append_edge(&mut fragments, c, a);
    fragments
}

fn filled_barycentric(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let p0 = a.transformed_position;
    let p1 = b.transformed_position;
    let p2 = c.transformed_position;
    let area = (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x);
    if area.abs() < f32::EPSILON {
        return Vec::new();
    }

    let min_x = (p0.x.min(p1.x).min(p2.x) - 0.5).ceil() as i32;
    let max_x = (p0.x.max(p1.x).max(p2.x) - 0.5).floor() as i32;
    let min_y = (p0.y.min(p1.y).min(p2.y) - 0.5).ceil() as i32;
    let max_y = (p0.y.max(p1.y).max(p2.y) - 0.5).floor() as i32;
    let mut fragments = Vec::new();
    let color = Vector3::new(1.0, 1.0, 1.0);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let w0 = ((p1.x - px) * (p2.y - py) - (p1.y - py) * (p2.x - px)) / area;
            let w1 = ((p2.x - px) * (p0.y - py) - (p2.y - py) * (p0.x - px)) / area;
            let w2 = 1.0 - w0 - w1;

            if w0 >= -1e-6 && w1 >= -1e-6 && w2 >= -1e-6 {
                fragments.push(Fragment::new(x as f32, y as f32, color, 0.0));
            }
        }
    }
    fragments
}

fn filled_scanline(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let vertices = [a, b, c];
    let min_y = (vertices
        .iter()
        .map(|v| v.transformed_position.y)
        .fold(f32::INFINITY, f32::min)
        - 0.5)
        .ceil() as i32;
    let max_y = (vertices
        .iter()
        .map(|v| v.transformed_position.y)
        .fold(f32::NEG_INFINITY, f32::max)
        - 0.5)
        .floor() as i32;
    let mut fragments = Vec::new();
    let color = Vector3::new(1.0, 1.0, 1.0);

    for y in min_y..=max_y {
        let scan_y = y as f32 + 0.5;
        let mut intersections: Vec<f32> = Vec::with_capacity(6);
        for (start, end) in [(a, b), (b, c), (c, a)] {
            let p0 = start.transformed_position;
            let p1 = end.transformed_position;
            if (p0.y - p1.y).abs() <= f32::EPSILON {
                if (scan_y - p0.y).abs() <= 1e-6 {
                    intersections.push(p0.x);
                    intersections.push(p1.x);
                }
            } else if scan_y >= p0.y.min(p1.y) - 1e-6
                && scan_y <= p0.y.max(p1.y) + 1e-6
            {
                let t = (scan_y - p0.y) / (p1.y - p0.y);
                intersections.push(p0.x + (p1.x - p0.x) * t);
            }
        }
        intersections.sort_by(f32::total_cmp);
        intersections.dedup_by(|left, right| (*left - *right).abs() <= 1e-5);
        if intersections.is_empty() {
            continue;
        }
        let left_x = intersections[0];
        let right_x = *intersections.last().unwrap();
        let first_x = (left_x - 0.5).ceil() as i32;
        let last_x = (right_x - 0.5).floor() as i32;
        for x in first_x..=last_x {
            fragments.push(Fragment::new(x as f32, y as f32, color, 0.0));
        }
    }
    fragments
}

pub fn triangle(a: &Vertex, b: &Vertex, c: &Vertex, mode: TriangleMode) -> Vec<Fragment> {
    match mode {
        TriangleMode::Wireframe => wireframe_triangle(a, b, c),
        TriangleMode::FilledBarycentric => filled_barycentric(a, b, c),
        TriangleMode::FilledScanline => filled_scanline(a, b, c),
    }
}
