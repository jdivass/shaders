use crate::fragment::Fragment;
use crate::triangle::{triangle, TriangleMode};
use crate::vertex::Vertex;

pub fn rasterize(triangles: &[[Vertex; 3]], mode: TriangleMode) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    for tri in triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], mode));
    }

    fragments
}
