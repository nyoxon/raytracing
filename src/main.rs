#![allow(dead_code, unused_variables)]

use nalgebra::{Vector3};
pub mod engine;

use engine::*;

fn main() {
    let sphere1 = Sphere {
        center: Vector3::new(0.0, 0.0, -5.0),
        radius: 1.0,
        color: (200.0, 200.0, 200.0),
        reflectivity: 1.0,
    };

    let sphere2 = Sphere {
        center: Vector3::new(-2.0, -0.5, 1.0),
        radius: 0.5,
        color: (255.0, 255.0, 0.0),
        reflectivity: 0.0,
    };

    let sphere3 = Sphere {
        center: Vector3::new(2.0, 0.75, 2.0),
        radius: 1.5,
        color: (255.0, 0.0, 0.0),
        reflectivity: 0.0,
    };

    let ground = Plane {
        point: Vector3::new(0.0, -1.0, 0.0),
        normal: Vector3::new(0.0, 1.0, 0.0),
        color: (0.0, 0.0, 255.0),
        reflectivity: 0.0,
    };
    
    let objects: Vec<Box<&dyn Intersectable>> = vec!
    [
        Box::new(&sphere1),
        Box::new(&sphere2),
        Box::new(&sphere3),
        Box::new(&ground),
    ];

    let camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0), //origin
        Vector3::new(0.0, 0.0, -1.0), //look_at
        Vector3::new(0.0, 1.0, 0.0), //up_hint
        1.0, // distance
        600, // height
        800, // width
        90.0, // fov
    );

    let ray_directions = camera.new_render
        (&objects, "output.ppm");
    println!("renderização concluída!");
}
