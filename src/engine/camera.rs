use nalgebra::Vector3;
use super::objects::types::{Ray, Intersection, LightSource};
use super::objects::traits::{Intersectable};
use std::fs::File;
use std::io::Write;

pub struct Camera {
    origin: Vector3<f32>,
    forward: Vector3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,
    distance: f32,
    height: usize,
    width: usize,
    fov: f32,
}

impl Camera {
    pub fn new
    (
        origin: Vector3<f32>,
        look_at: Vector3<f32>,
        up_hint: Vector3<f32>,
        distance: f32,
        height: usize,
        width: usize,
        fov: f32, 
    ) -> Self {
        let forward = (look_at - origin).normalize();
        let right = forward.cross(&up_hint).normalize();
        let up = right.cross(&forward).normalize();

        Self {
            origin,
            forward,
            right,
            up,
            distance,
            height,
            width,
            fov,
        }
    }
}

impl Camera {
    pub fn generate_rays(&self) -> Vec<Ray> {
        let aspect_ratio = self.width as f32 / self.height as f32;
        let fov_rad = self.fov.to_radians();

        let image_plane_height = 2.0 * (fov_rad / 2.0)
                .tan() * self.distance;

        let image_plane_width = image_plane_height
            * aspect_ratio;

        let mut rays = vec![];

        for j in 0..self.height {
            for i in 0..self.width {
                let pixel_x = (i as f32 + 0.5)
                    / self.width as f32;
                let pixel_y = (j as f32 + 0.5)
                    / self.height as f32;

                let pixel_screen_x = (2.0 *
                    pixel_x - 1.0) * image_plane_width
                    / 2.0;
                let pixel_screen_y = (1.0 -
                    2.0 * pixel_y) * image_plane_height
                    / 2.0;

                let pixel_position = self.origin
                    + self.forward * self.distance
                    + self.right * pixel_screen_x
                    + self.up * pixel_screen_y;

                let ray_direction = (pixel_position - self.origin).normalize();
                let ray =  Ray {
                    origin: self.origin,
                    direction: ray_direction,
                };
                
                rays.push(ray);
            }
        }

        rays
    }

    pub fn trace_ray
    (
        &self,
        ray: &Ray,
        objects: &Vec<Box<&dyn Intersectable>>,
        depth: u32,
    ) -> (u8, u8, u8) {
        if depth == 0 {
            return (0, 0, 0);
        }

        let mut closest: Option<Intersection> = None;
        let mut min_dist = f32::MAX;

        for obj in objects  {
            if let Some(hit) = obj.intersect(ray) {
                if hit.distance < min_dist {
                    min_dist = hit.distance;
                    closest = Some(hit);
                }
            }
        }

        if let Some(hit) = closest {
            let light = LightSource {
                origin: Vector3::new(-5.0, 5.0, 5.0),
                intensity: (255.0, 255.0, 255.0),
                color: (255.0, 0.0, 100.0),
            };

            let light_dir = (light.origin - hit.point).normalize();
            let normal = hit.normal;
            let product = normal.dot(&light_dir).max(0.0);

            let mut intensity = ((light.intensity.0 / 255.0 * product).clamp(0.0, 1.0),
                (light.intensity.1 / 255.0 * product).clamp(0.0, 1.0),
                (light.intensity.2 / 255.0 * product).clamp(0.0, 1.0));

            let light_color = ((light.color.0 / 255.0).clamp(0.0, 1.0),
                (light.color.1 / 255.0).clamp(0.0, 1.0),
                (light.color.2 / 255.0).clamp(0.0, 1.0)
            );

            let shadow_ray = Ray {
                origin: hit.point + normal * 1e-3,
                direction: light_dir,
            };


            let base_color = hit.object.get_color();

            for obj in objects {
                if let Some(shadow_hit) = obj.intersect(&shadow_ray) {
                    if shadow_hit.distance > 1e-3 {
                        intensity = (0.0, 0.0, 0.0);
                        break;
                    } 
                }
            }

            let local_color = (
                (base_color.0 as f32 / 255.0 * intensity.0).clamp(0.0, 1.0),
                (base_color.1 as f32 / 255.0 * intensity.1).clamp(0.0, 1.0),
                (base_color.2 as f32 / 255.0 * intensity.2).clamp(0.0, 1.0),
            );

            let local_color = (
                (local_color.0.clamp(0.0, 1.0) * light_color.0.clamp(0.0, 1.0) * 255.0) as u8,
                (local_color.1.clamp(0.0, 1.0) * light_color.1.clamp(0.0, 1.0) * 255.0) as u8,
                (local_color.2.clamp(0.0, 1.0) * light_color.2.clamp(0.0, 1.0) * 255.0) as u8,
            );


            let reflectivity = hit.object.reflectivity();

            if reflectivity > 0.0 {
                let reflected_dir = self.reflect(
                    &ray.direction, &normal
                ).normalize();
                let reflected_ray = Ray {
                    origin: hit.point + normal * 1e-3,
                    direction: reflected_dir,
                };

                let reflected_color = self.trace_ray(
                    &reflected_ray, objects, depth - 1
                );

                return (
                    (local_color.0 as f32 * (1.0 - reflectivity)
                     + reflected_color.0 as f32 * reflectivity) as u8,
                    (local_color.1 as f32 * (1.0 - reflectivity)
                     + reflected_color.1 as f32 * reflectivity) as u8,
                    (local_color.2 as f32 * (1.0 - reflectivity)
                     + reflected_color.2 as f32 * reflectivity) as u8,
                );
            }

            return local_color
        }

        (0, 0, 0)
    }

    pub fn closest_intersection<'a>
    (
        &self,
        ray: &Ray,
        objects: &'a Vec<Box<&'a dyn Intersectable>>,
    ) -> Option<(&'a dyn Intersectable, Intersection<'a>)> {
        let mut closest_t = f32::MAX;
        let mut result: Option<
        (
            &'a dyn Intersectable,
            Intersection<'a>
        )> = None;

        for object in objects {
            if let Some(intersection) =
                object.intersect(&ray) {
                let hit_point = intersection.point;
                let normal = intersection.normal;
                let t = intersection.distance;

                if t < closest_t {
                    closest_t = t;
                    result = Some((**object, intersection));
                }
            }
        }

        result
    }

    pub fn new_render
    (
        &self,
        objects: &Vec<Box<&dyn Intersectable>>,
        filename: &str
    ) {
        let mut file = File::create(filename)
            .expect("Error ao criar arquivo");

        writeln!(file, "P3").unwrap();
        writeln!(file, "{} {}", self.width, self.height).unwrap();
        writeln!(file, "255").unwrap();

        let rays = self.generate_rays();

        for ray in &rays {
            let color = self.trace_ray(
                &ray, objects, 3);


            writeln!(file, "{} {} {}",
                color.0, color.1, color.2).unwrap();
        }
    }

    pub fn reflect
    (
        &self,
        incident: &Vector3<f32>,
        normal: &Vector3<f32>
    ) -> Vector3<f32> {
        incident - &(2.0 * (incident.dot(normal)) * normal)
    }

}