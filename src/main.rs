#![allow(dead_code, unused_variables)]

use nalgebra::{Vector3};
use std::fs::File;
use std::io::Write;

trait Intersectable {
    fn intersect
    (
        &self,
        ray: &Ray
    ) -> Option<Intersection>;

    fn get_color
    (
        &self,
    ) -> (f32, f32, f32);

    fn reflectivity(&self) -> f32;
}

struct Intersection<'a> {
    point: Vector3<f32>,
    normal: Vector3<f32>,
    distance: f32,
    object: &'a dyn Intersectable
}

struct LightSource {
    origin: Vector3<f32>,
    intensity: (f32, f32, f32),
    color: (f32, f32, f32),
}

struct Ray {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
}

struct Sphere {
    center: Vector3<f32>,
    radius: f32,
    color: (f32, f32, f32),
    reflectivity: f32,
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let oc = ray.origin - self.center;

        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;

        let discriminant: f32 = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);

        let t = if t1 >= 0.0 {
            t1
        } else if t2 >= 0.0 {
            t2
        } else {
            return None;
        };

        let point = ray.origin + t * ray.direction;
        let normal = (point - self.center).normalize();

        Some(Intersection {
            point,
            normal,
            distance: t,
            object: self,
        })
    }

    fn get_color(&self) -> (f32, f32, f32) {
        self.color
    }

    fn reflectivity(&self) -> f32 {
        self.reflectivity
    }
}

struct Plane {
    point: Vector3<f32>,
    normal: Vector3<f32>,
    color: (f32, f32, f32),
    reflectivity: f32,
}

struct RayTracing {
    origin: Vector3<f32>,
    forward: Vector3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,
    distance: f32,
    height: usize,
    width: usize,
    fov: f32,
}

impl RayTracing {
    fn new
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

impl Intersectable for Plane {
    
    fn intersect
    (
        &self,
        ray: &Ray,
    ) -> Option<Intersection> {
        let denom = ray.direction.dot(&self.normal);

        if denom.abs() < 1e-6 {
            return None;
        }

        let t = (self.point - ray.origin).
                dot(&self.normal) / denom;

        if t < 0.0 {
            return None;
        }

        let hit_point = ray.origin + t * ray.direction;
        Some(Intersection {
            point: hit_point,
            normal: self.normal,
            distance: t,
            object: self,
        })
    }

    fn get_color(&self) -> (f32, f32, f32) {
        self.color
    }

    fn reflectivity(&self) -> f32 {
        self.reflectivity
    }
}

impl RayTracing {
    fn generate_rays(&self) -> Vec<Ray> {
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

    fn trace_ray
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

    fn closest_intersection<'a>
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

    fn new_render
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

    fn reflect
    (
        &self,
        incident: &Vector3<f32>,
        normal: &Vector3<f32>
    ) -> Vector3<f32> {
        incident - &(2.0 * (incident.dot(normal)) * normal)
    }

}

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

    let ray_tracing = RayTracing::new(
        Vector3::new(0.0, 0.0, 5.0), //origin
        Vector3::new(0.0, 0.0, -1.0), //look_at
        Vector3::new(0.0, 1.0, 0.0), //up_hint
        1.0, // distance
        600, // height
        800, // width
        90.0, // fov
    );

    let ray_directions = ray_tracing.new_render
        (&objects, "output.ppm");
    println!("renderização concluída!");
}
