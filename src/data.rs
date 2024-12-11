use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::Serialize;
use crate::rtweekend::{AlgorithmOptions, IntersectionAlgorithm, Options};

#[derive(Default, Serialize)]
pub struct Data {
    seconds: f64,
    init_time: f64,
    primary_rays: usize,
    scatter_rays: usize,
    intersection_checks: usize,
    traversal_steps: usize,
    overlapping_aabb: usize,
    filename: String,
    primitives: usize,
    algorithm: IntersectionAlgorithm,
    options: Options,
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
    max_depth: u32,
}

impl Data {
    #[allow(clippy::too_many_arguments)]
    pub fn new(filename: String, primitives: usize, algorithm: IntersectionAlgorithm, options: Options,
               image_width: u32, image_height: u32, samples_per_pixel: u32, max_depth: u32) -> Self {
        Self {
            filename,
            primitives,
            algorithm,
            options,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
            ..Default::default()
        }
    }

    pub fn print(&self) {
        println!("Total primary rays: {}", self.primary_rays());
        println!("Total scatter rays: {}", self.scatter_rays());
        println!("Overlapping AABBs: {}", self.overlapping_aabb());
        println!("Total intersection checks: {}", self.intersection_checks());
        println!("Total traversal steps: {}", self.traversal_steps());
        println!("Total init time: {}", self.init_time());
        println!("Total time: {}", self.seconds());
    }

    pub fn write_to_csv(&self, filename: &PathBuf) {
        let file_exists = Path::new(filename).exists();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .expect(&format!("Cannot open {filename:?}"));


        let mut writer = csv::WriterBuilder::new()
            .has_headers(!file_exists) // Write headers only if the file doesn't exist
            .from_writer(file);

        writer.serialize(self).expect("Failed to serialise CSV data");
        writer.flush().expect("Failed to write CSV data");
    }
    
    pub fn seconds(&self) -> f64 {
        self.seconds
    }

    pub fn set_seconds(&mut self, seconds: f64) {
        self.seconds = seconds;
    }

    pub fn init_time(&self) -> f64 {
        self.init_time
    }

    pub fn set_init_time(&mut self, seconds: f64) {
        self.init_time = seconds;
    }
    pub fn primary_rays(&self) -> usize {
        self.primary_rays
    }

    pub fn add_primary_ray(&mut self) {
        self.primary_rays += 1;
    }

    pub fn scatter_rays(&self) -> usize {
        self.scatter_rays
    }

    pub fn add_scatter_ray(&mut self) {
        self.scatter_rays += 1;
    }

    pub fn intersection_checks(&self) -> usize {
        self.intersection_checks
    }

    pub fn add_intersection_check(&mut self) {
        self.intersection_checks += 1;
    }

    pub fn traversal_steps(&self) -> usize {
        self.traversal_steps
    }

    pub fn add_traversal_step(&mut self) {
        self.traversal_steps += 1;
    }

    pub fn overlapping_aabb(&self) -> usize {
        self.overlapping_aabb
    }

    pub fn add_overlapping_aabb(&mut self) {
        self.overlapping_aabb += 1;
    }
}