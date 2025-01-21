use crate::camera::Camera;
use crate::gpu::state::GPUState;
use clap::Parser;
use std::fs::File;
use std::rc::Rc;
use std::time::Instant;
use utils::parser::parse_ply;
use utils::rtweekend::{check_valid_options, get_output_filename, Cli, FileFormat, Options};
use utils::scenes;
use value::color::Color;
use value::data::Data;
use value::material::{Lambertian, MaterialType};
use value::vec3::{Point3, Vec3};

mod hittable;
mod camera;
mod acceleration;
mod value;
mod utils;
mod gpu;
#[cfg(test)]
mod test;

fn main() {
    // Parse CLI arguments
    let args = Cli::parse();
    if let Some(error) = check_valid_options(&args.options) {
        panic!("{error}")
    }
    // Run async function using pollster
    pollster::block_on(
        run(args)
    );
}

async fn run(args: Cli) {
    let options = Options::new(&args);

    let (mut world, filename) = if let Some(filename) = args.filename {
        match args.format {
            FileFormat::Native => {
                // Deserialize the object
                let file = File::open(&filename).expect("Could not open scene file");
                let world = serde_json::from_reader(&file).expect("Could not read scene file");
                (world, filename)
            }
            FileFormat::PLY => {
                let material = Rc::new(Lambertian::new(Color::new(0.8, 0.2, 0.1)));
                let world = parse_ply(&filename.clone().into(), material).expect("Failed to open PLY scene");
                (world, filename)
            }
        }
    } else {
        // let (world, filename) = scenes::weekend_final();
        // let (world, filename) = scenes::weekend_custom(2, 0.9, 0.05);
        // let (world, filename) = scenes::weekend_custom(1, 0.5, 0.25);
        // let (world, filename) = scenes::weekend_custom(5, 0.8, 0.15);
        let (world, filename) = scenes::simple_diffuse();
        // let (world, filename) = scenes::simple_hollow_glass();
        // let (world, filename) = scenes::simple_shiny_metal();
        // let (world, filename) = scenes::simple_fuzzy_metal();
        // let (world, filename) = scenes::simple_triangle();
        // let (world, filename) = scenes::triangle_materials();
        // let (world, filename) = scenes::triangle_test();

        // Serialize the world
        let filename = format!("scenes/{filename}.json");
        let file = File::create(&filename).expect("Could not open scene file");
        serde_json::to_writer(&file, &world).expect("Could not write to scene file");
        eprintln!("Wrote scene to {filename}");
        // Return world
        (world, filename)
    };

    world.algorithm = args.algorithm;
    world.options = options.clone();

    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1024;
    cam.samples_per_pixel = 50;
    cam.max_depth = 50;
    cam.defocus_angle = 0.1;
    cam.focus_dist = 1.0;

    // TODO this is very hacky, encode this in the json files
    if filename.starts_with("scenes/weekend") {
        cam.vfov = 20.0;
        cam.look_from = Point3::new(13.0, 2.0, 3.0);
        cam.look_at = Point3::new(0.0, 0.0, 0.0);
    } else if filename.starts_with("scenes/dragon") {
        cam.vfov = 20.0;
        cam.focus_dist = 50.0;
        cam.look_at = Point3::new(0.0, 12.0, 0.0);
        cam.look_from =
            match args.camera {
                None | Some(0) => { Point3::new(0.0, 15.0, 50.0) }
                Some(1) => { Point3::new(-50.0, 15.0, 20.0) }
                Some(2) => { Point3::new(80.0, 15.0, 10.0) }
                Some(3) => { Point3::new(10.0, 50.0, 25.0) }
                Some(4) => { Point3::new(-10.0, 30.0, 25.0) }
                Some(_) => {panic!("Camera position does not exist")}
            };
    } else {
        cam.vfov = 90.0;
        cam.look_from = Point3::new(0.0, 0.0, 0.0);
        // cam.look_from = Point3::new(1.0, 1.0, 1.0);
        cam.look_at = Point3::new(0.0, 0.0, -1.0);
    }
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    // Scene statistics
    if args.stats {
        let lambertian_materials = world.objects.iter().filter(|i| i.material_type() == Some(MaterialType::Lambertian)).count();
        let metal_materials = world.objects.iter().filter(|i| i.material_type() == Some(MaterialType::Metal)).count();
        let dielectric_materials = world.objects.iter().filter(|i| i.material_type() == Some(MaterialType::Dielectric)).count();

        println!("Name & \\# Primitives & \\# Lambertian primitives & \\# Metal primitives & \\# Dieelectric primitives \\\\");
        println!("{filename} & {} & {} & {} & {}\\\\",
                 world.objects.len(), lambertian_materials, metal_materials, dielectric_materials);
        return;
    }

    // Open file
    let out_filename = get_output_filename(&filename, &world.algorithm, &options)
        .expect("Could not parse filename");
    let mut file = File::create(&out_filename)
        .expect("Could not open image file");

    let mut data: Data = Data::new(filename.to_string(), world.objects.len(), args.algorithm, options, cam.image_width, cam.image_height(), cam.samples_per_pixel, cam.max_depth);
    let start = Instant::now();

    if args.gpu { 
        // GPU rendering
        let state = GPUState::new(&mut cam, &world).await;
        data.set_init_time(start.elapsed().as_secs_f64());
        state.render(&mut file).await
            .expect("Could not write to image file");
    } else {
        // CPU rendering
        // Initialise structures like BVH
        world.init();
        data.set_init_time(start.elapsed().as_secs_f64());

        // Render pixels
        cam.render(&world, &mut file, &mut data)
            .expect("Could not write to image file");
    }
    
    data.set_seconds(start.elapsed().as_secs_f64());

    data.print();
    data.write_to_csv(&"output/stats.csv".into());

    eprintln!("Wrote image to {out_filename}. Duration {:3.2?}", start.elapsed());
}

