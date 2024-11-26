use crate::camera::Camera;
use crate::vec3::{Point3, Vec3};
use clap::Parser;
use std::fs::File;
use std::time::Instant;
use crate::data::Data;
use crate::rtweekend::{get_output_filename, IntersectionAlgorithm};

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
mod triangle;
mod data;
mod grid;
mod intbox;
mod intpoint;

#[derive(Parser)]
struct Cli {
    /// The world / scene file
    filename: Option<String>,
    #[arg(long, default_value_t = IntersectionAlgorithm::default())]
    /// The intersection algorithm
    algorithm: IntersectionAlgorithm
}

fn main() {
    // Parse CLI arguments
    let args = Cli::parse();

    let (world, filename) = if let Some(filename) = args.filename {
        // Deserialize the object
        let file = File::open(&filename).expect("Could not open scene file");
        let world = serde_json::from_reader(&file).expect("Could not read scene file");
        (world, filename)
    } else {
        // let (world, filename) = scenes::weekend_final();
        // let (world, filename) = scenes::weekend_custom(2, 0.9, 0.05);
        // let (world, filename) = scenes::weekend_custom(1, 0.5, 0.25);
        // let (world, filename) = scenes::weekend_custom(5, 0.8, 0.15);
        // let (world, filename) = scenes::simple_hollow_glass();
        // let (world, filename) = scenes::simple_shiny_metal();
        // let (world, filename) = scenes::simple_fuzzy_metal();
        // let (world, filename) = scenes::simple_triangle();
        let (world, filename) = scenes::triangle_materials();

        // Serialize the world
        let filename = format!("scenes/{filename}.json");
        let file = File::create(&filename).expect("Could not open scene file");
        serde_json::to_writer(&file, &world).expect("Could not write to scene file");
        eprintln!("Wrote scene to {filename}");
        // Return world
        (world, filename)
    };


    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 900;
    cam.samples_per_pixel = 50;
    cam.max_depth = 50;

    cam.vfov = 90.0;
    cam.look_from = Point3::new(0.0, 0.0, 0.0);
    cam.look_at = Point3::new(0.0, 0.0, -1.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.1;
    cam.focus_dist = 1.0;
    
    // Open file
    let filename = get_output_filename(&filename)
        .expect("Could not parse filename");
    let mut file = File::create(&filename)
        .expect("Could not open image file");

    let mut data: Data = Data::new();

    let start = Instant::now();
    cam.render(&world, &mut file, &mut data)
        .expect("Could not write to image file");
    eprintln!("Wrote image to {filename}. Duration {:3.2?}", start.elapsed());
    data.set_seconds(start.elapsed().as_secs_f64());
    println!("Total primary rays: {}", data.primary_rays());
    println!("Total scatter rays: {}", data.scatter_rays());
    println!("Total intersection checks: {}", data.intersection_checks());
    println!("Total seconds: {}", data.seconds());
}

