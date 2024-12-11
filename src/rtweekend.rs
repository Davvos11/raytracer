use std::fmt::{Display, Formatter};
use std::path::Path;
use clap::{Parser, ValueEnum};
use rand::Rng;
use serde::{Serialize, Serializer};
use crate::rtweekend::AlgorithmOptions::{BvhNaive, BvhSahPlane, BvhSahPosition};

#[derive(Parser, Default)]
pub struct Cli {
    /// The world / scene file
    pub filename: Option<String>,
    #[arg(long, value_enum, default_value_t = FileFormat::default())]
    /// The input file format
    pub format: FileFormat,
    #[arg(long, value_enum, default_value_t = IntersectionAlgorithm::default())]
    /// The intersection algorithm
    pub algorithm: IntersectionAlgorithm,
    /// Options for the algorithm
    #[arg(value_enum, long, short)]
    pub options: Vec<AlgorithmOptions>,
    /// Grid size (if algorithm is grid)
    #[arg(long, short, default_value_t = 25.0)]
    pub grid_size: f64,
    /// Print scene statistics (as LaTeX table row) and exit
    #[arg(long)]
    pub stats: bool,
    /// Camera position (only for dragon scene)
    #[arg(long)]
    pub camera: Option<usize>,
}

#[allow(unused)]
impl Cli {
    pub fn new_from_json(filename: String) -> Self{
        Self {
            filename: Some(filename),
            format: FileFormat::Native,
            ..Default::default()
        }
    }

    pub fn new_from_ply(filename: String) -> Self{
        Self {
            filename: Some(filename),
            format: FileFormat::PLY,
            ..Default::default()
        }
    }

    pub fn add_option(&mut self, option: AlgorithmOptions) {
        self.options.push(option);
    }
}

#[derive(Default, Copy, Clone, ValueEnum, Serialize, Debug, PartialEq)]
pub enum IntersectionAlgorithm {
    Naive,
    #[default]
    BVH,
    Grid
}

#[derive(Default, Copy, Clone, ValueEnum)]
pub enum FileFormat {
    #[default]
    Native,
    PLY,
}

#[derive(Default, Clone, Debug)]
pub struct Options {
    pub algorithm: IntersectionAlgorithm,
    pub options: Vec<AlgorithmOptions>,
    pub draw_boxes: bool,
    pub grid_size: f64,
    pub camera: Option<usize>,
}

impl Display for Options {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut option_strs = self.options.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>();
        if self.algorithm == IntersectionAlgorithm::Grid {
            option_strs.push(format!("size={}", self.grid_size));
        }
        if let Some(pos) = self.camera {
            option_strs.push(format!("pos{}", pos));
        }
        let joined = option_strs.join("_");
        f.write_str(&joined)
    }
}

impl Serialize for Options {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl Options {
    pub fn new(args: &Cli) -> Self {
        Self {
            algorithm: args.algorithm,
            draw_boxes: args.options.contains(&AlgorithmOptions::DrawBoxes),
            options: args.options.clone(),
            grid_size: args.grid_size,
            camera: args.camera,
        }
    }
}

#[derive(Copy, Clone, ValueEnum, Debug, Eq, PartialEq, Serialize)]
pub enum AlgorithmOptions {
    // BVH options:
    /// Naive BVH, always split on the x plane and position halfway
    BvhNaive,
    /// BVH with SAH but only to determine the plane, not the split position
    BvhSahPlane,
    /// BVH with SAH for the plane and the split position (default)
    BvhSahPosition,
    /// Draw bounding boxes
    DrawBoxes,
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
            IntersectionAlgorithm::Naive => {write!(f, "naive")}
            IntersectionAlgorithm::BVH => {write!(f, "bvh")}
            IntersectionAlgorithm::Grid => {write!(f, "grid")}
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

pub fn get_output_filename(input_path: &String, algorithm: &IntersectionAlgorithm, options: &Options) -> Option<String> {
    let path = Path::new(input_path);
    // Extract the file stem (name without extension)
    if let Some(stem) = path.file_stem() {
        // Construct the new file name with the desired extension
        let options_str = options.to_string();
        let options_str = if options_str.is_empty() { "" } else { &format!("-{options_str}")};
        let new_file_name = format!("output/{}-{algorithm}{options_str}.ppm", stem.to_string_lossy());
        return Some(new_file_name);
    }
    None
}