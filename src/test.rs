use crate::rtweekend::{AlgorithmOptions, IntersectionAlgorithm};
use crate::{run, Cli};

#[test]
fn test_simple_fuzzy_metal_naive() {
    let mut settings = Cli::new_from_json("scenes/simple_fuzzy_metal.json".to_string());
    settings.algorithm = IntersectionAlgorithm::Naive;
    run(settings);
}
#[test]
fn test_simple_fuzzy_metal_bvh_naive() {
    let mut settings = Cli::new_from_json("scenes/simple_fuzzy_metal.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhNaive);
    run(settings);
}
#[test]
fn test_simple_fuzzy_metal_bvh_sah_plane() {
    let mut settings = Cli::new_from_json("scenes/simple_fuzzy_metal.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPlane);
    run(settings);
}
#[test]
fn test_simple_fuzzy_metal_bvh_sah_full() {
    let mut settings = Cli::new_from_json("scenes/simple_fuzzy_metal.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPosition);
    run(settings);
}


#[test]
fn test_triangle_materials_naive() {
    let mut settings = Cli::new_from_json("scenes/triangle_materials.json".to_string());
    settings.algorithm = IntersectionAlgorithm::Naive;
    run(settings);
}
#[test]
fn test_triangle_materials_bvh_naive() {
    let mut settings = Cli::new_from_json("scenes/triangle_materials.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhNaive);
    run(settings);
}
#[test]
fn test_triangle_materials_bvh_sah_plane() {
    let mut settings = Cli::new_from_json("scenes/triangle_materials.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPlane);
    run(settings);
}
#[test]
fn test_triangle_materials_bvh_sah_full() {
    let mut settings = Cli::new_from_json("scenes/triangle_materials.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPosition);
    run(settings);
}

#[test]
fn test_many_balls_naive() {
    let mut settings = Cli::new_from_json("scenes/weekend-5-80-15.json".to_string());
    settings.algorithm = IntersectionAlgorithm::Naive;
    run(settings);
}
#[test]
fn test_many_balls_bvh_naive() {
    let mut settings = Cli::new_from_json("scenes/weekend-5-80-15.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhNaive);
    run(settings);
}
#[test]
fn test_many_balls_bvh_sah_plane() {
    let mut settings = Cli::new_from_json("scenes/weekend-5-80-15.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPlane);
    run(settings);
}
#[test]
fn test_many_balls_bvh_sah_full() {
    let mut settings = Cli::new_from_json("scenes/weekend-5-80-15.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPosition);
    run(settings);
}

#[test]
fn test_weekend_final_bvh_naive() {
    let mut settings = Cli::new_from_json("scenes/weekend-final.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhNaive);
    run(settings);
}
#[test]
fn test_weekend_final_bvh_sah_plane() {
    let mut settings = Cli::new_from_json("scenes/weekend-final.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPlane);
    run(settings);
}
#[test]
fn test_weekend_final_bvh_sah_full() {
    let mut settings = Cli::new_from_json("scenes/weekend-final.json".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPosition);
    run(settings);
}

#[test]
fn test_dragon_4_bvh_naive() {
    let mut settings = Cli::new_from_ply("scenes/dragon_recon/dragon_vrip_res4.ply".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhNaive);
    run(settings);
}
#[test]
fn test_dragon_4_bvh_sah_plane() {
    let mut settings = Cli::new_from_ply("scenes/dragon_recon/dragon_vrip_res4.ply".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPlane);
    run(settings);
}
#[test]
fn test_dragon_4_bvh_sah_full() {
    let mut settings = Cli::new_from_ply("scenes/dragon_recon/dragon_vrip_res4.ply".to_string());
    settings.algorithm = IntersectionAlgorithm::BVH;
    settings.add_option(AlgorithmOptions::BvhSahPosition);
    run(settings);
}
