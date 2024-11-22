use crate::camera::Camera;
use crate::vec3::{Point3, Vec3};
use clap::Parser;
use std::fs::File;

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
mod scenes;

#[derive(Parser)]
struct Cli {
    /// The world / scene file
    filename: Option<String>,
}

fn main() {
    // Parse CLI arguments
    let args = Cli::parse();

    let world = if let Some(filename) = args.filename {
        // Deserialize the object
        let file = File::open(filename).expect("Could not open file");
        serde_json::from_reader(&file).expect("Could not read file")
    } else {
        // let (world, filename) = scenes::weekend_final();
        // let (world, filename) = scenes::weekend_custom(2, 0.9, 0.05);
        // let (world, filename) = scenes::weekend_custom(1, 0.5, 0.25);
        // let (world, filename) = scenes::weekend_custom(5, 0.8, 0.15);
        // let (world, filename) = scenes::simple_hollow_glass();
        // let (world, filename) = scenes::simple_shiny_metal();
        let (world, filename) = scenes::simple_fuzzy_metal();

        // Serialize the world
        let filename = format!("scenes/{filename}.json");
        let file = File::create(&filename).expect("Could not open file");
        serde_json::to_writer(&file, &world).expect("Could not write to file");
        eprintln!("Wrote scene to {filename}");
        // Return world
        world
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

