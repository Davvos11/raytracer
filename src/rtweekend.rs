use std::path::Path;
use rand::Rng;

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

pub fn get_output_filename(input_path: &String) -> Option<String> {
    let path = Path::new(input_path);
    // Extract the file stem (name without extension)
    if let Some(stem) = path.file_stem() {
        // Construct the new file name with the desired extension
        let new_file_name = format!("output/{}.ppm", stem.to_string_lossy());
        return Some(new_file_name);
    }
    None
}