use super::types::{Ray, Intersection, Sphere, Plane};

pub trait Intersectable {
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