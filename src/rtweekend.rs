use std::fmt::{Display, Formatter};
use std::path::Path;
use clap::ValueEnum;
use rand::Rng;
use crate::rtweekend::AlgorithmOptions::{BvhNaive, BvhSahPlane, BvhSahPosition};

#[derive(Default, Copy, Clone, ValueEnum)]
pub enum IntersectionAlgorithm {
    Naive,
    #[default]
    BVH,
}

#[derive(Default, Copy, Clone, ValueEnum)]
pub enum FileFormat {
    #[default]
    Native,
    PLY,
}

#[derive(Default, Clone,Debug, Eq, PartialEq)]
pub struct Options {
    pub options: Vec<AlgorithmOptions>,
    pub draw_boxes: bool,
}

impl Options {
    pub fn new(alg_options: Vec<AlgorithmOptions>) -> Self {
        Self {
            draw_boxes: alg_options.contains(&AlgorithmOptions::DrawBoxes),
            options: alg_options,
        }
    }
}

#[derive(Copy, Clone, ValueEnum, Debug, Eq, PartialEq)]
pub enum AlgorithmOptions {
    // BVH options:
    /// Naive BVH, always split on the x plane and position halfway
    BvhNaive,
    /// BVH with SAH but only to determine the plane, not the split position
    BvhSahPlane,
    /// BVH with SAH for the plane and the split position (default)
    BvhSahPosition,
    /// Draw bounding boxes
    DrawBoxes
}
const BVH_OPTIONS: &[AlgorithmOptions] = &[BvhNaive, BvhSahPlane, BvhSahPosition];

pub fn check_valid_options(options: &[AlgorithmOptions]) -> Option<String> {
    // Only one of the different Bvh options is allowed
    let bvh_options = options.iter().filter(|&x| BVH_OPTIONS.contains(x));
    if bvh_options.clone().count() > 1 {
        return Some(format!("Can't have the following options at the same time: {:?}", bvh_options.collect::<Vec<_>>()));
    }
    None
}

impl Display for IntersectionAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IntersectionAlgorithm::Naive => { write!(f, "naive") }
            IntersectionAlgorithm::BVH => { write!(f, "bvh") }
        }
    }
}

pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

pub fn random_double() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn random_double_range(min: f64, max: f64) -> f64 {
    min + (max - min) * random_double()
}

pub fn get_output_filename(input_path: &String, algorithm: &IntersectionAlgorithm) -> Option<String> {
    let path = Path::new(input_path);
    // Extract the file stem (name without extension)
    if let Some(stem) = path.file_stem() {
        // Construct the new file name with the desired extension
        let new_file_name = format!("output/{}-{algorithm}.ppm", stem.to_string_lossy());
        return Some(new_file_name);
    }
    None
}