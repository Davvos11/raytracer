use std::fs::File;
use crate::camera::Camera;
use crate::color::Color;
use crate::hittable_list::HittableList;
use crate::material::{Dielectric, Lambertian, Material, Metal};
use crate::sphere::Sphere;
use crate::vec3::{Point3, Vec3};
use std::rc::Rc;
use clap::Parser;
use crate::hittable::Hittable;
use crate::rtweekend::{random_double, random_double_range};

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

#[derive(Parser)]
struct Cli {
    /// The input / output file for the scene / world
    filename: String,
    /// Whether to generate a scene file
    #[arg(long, short, default_value_t = false)]
    write: bool,
}

fn main() {
    // Parse CLI arguments
    let args = Cli::parse();

    let world = if args.write {
        let mut world = HittableList::default();
        let ground_material = Rc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        world.add(Rc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material)));
        // Specify more elements of the scene here

        // Serialize the world
        let file = File::create(args.filename).expect("Could not open file");
        serde_json::to_writer(&file, &world).expect("Could not write to file");

        world
    } else {
        // Deserialize the object
        let file = File::open(args.filename).expect("Could not open file");
        serde_json::from_reader(&file).expect("Could not read file")
    };


    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 500;
    cam.max_depth = 50;

    cam.vfov = 20.0;
    cam.look_from = Point3::new(13.0, 2.0, 3.0);
    cam.look_at = Point3::new(0.0, 0.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.6;
    cam.focus_dist = 10.0;

    cam.render(&world);
}

