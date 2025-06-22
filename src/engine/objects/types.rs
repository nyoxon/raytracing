use nalgebra::{Vector3};
use super::traits::{Intersectable};

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub struct Intersection<'a> {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub distance: f32,
    pub object: &'a dyn Intersectable
}

pub struct LightSource {
    pub origin: Vector3<f32>,
    pub intensity: (f32, f32, f32),
    pub color: (f32, f32, f32),
}

pub struct Sphere {
    pub center: Vector3<f32>,
    pub radius: f32,
    pub color: (f32, f32, f32),
    pub reflectivity: f32,
}

pub struct Plane {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub color: (f32, f32, f32),
    pub reflectivity: f32,
}