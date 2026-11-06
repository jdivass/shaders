use raylib::prelude::*;

pub fn print_vertex(v: Vector3) {
    println!("x = {}",v.x);
    println!("y = {}",v.y);
    println!("z = {}",v.z);
}

pub fn print_point(v: Vector2) {
    println!("x = {}",v.x);
    println!("y = {}",v.y);
}
