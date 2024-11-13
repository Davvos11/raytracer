use crate::camera::Camera;
use crate::color::Color;
use crate::hittable_list::HittableList;
use crate::material::Lambertian;
use crate::sphere::Sphere;
use crate::vec3::Point3;
use std::f64::consts::PI;
use std::rc::Rc;

mod vec3;
mod color;
mod ray;
mod hittable;
mod sphere;
mod hittable_list;
mod rtweekend;
mod interval;
mod camera;
mod material;

fn main() {
    // World setup
    let mut world = HittableList::default();

    let r = (PI / 4.0).cos();

    let material_left = Rc::new(Lambertian::new(Color::new(0.0, 0.0, 1.0)));
    let material_right = Rc::new(Lambertian::new(Color::new(1.0, 0.0, 0.0)));

    world.add(Rc::new(Sphere::new(Point3::new(-r, 0.0, -1.0), r, material_left)));
    world.add(Rc::new(Sphere::new(Point3::new(r, 0.0, -1.0), r, material_right)));

    // Camera setup
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.vfov = 90.0;

    cam.render(&world);
}

