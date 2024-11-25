use std::rc::Rc;
use crate::color::Color;
use crate::hittable_list::HittableList;
use crate::material::{Dielectric, Lambertian, Metal};
use crate::rtweekend::{random_double, random_double_range};
use crate::sphere::Sphere;
use crate::triangle::Triangle;
use crate::vec3::Point3;

#[allow(dead_code)]
pub fn weekend_final() -> (HittableList, String) {
    let (world, _) = weekend_custom(11, 0.8, 0.15);
    (world, "weekend-final".to_string())
}

#[allow(dead_code)]
pub fn weekend_custom(small_sphere_multiplier: i32, diffuse_prob: f64, mat_prob: f64) -> (HittableList, String) {
    let mut world = HittableList::default();

    let ground_material = Rc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Rc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material)));

    for a in -small_sphere_multiplier..small_sphere_multiplier {
        for b in -small_sphere_multiplier..small_sphere_multiplier {
            let choose_mat = random_double();
            let center = Point3::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 * 0.9 + random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < diffuse_prob {
                    // Diffuse
                    let albedo = Color::random() * Color::random();
                    let material = Rc::new(Lambertian::new(albedo));
                    world.add(Rc::new(Sphere::new(center, 0.2, material)));
                } else if choose_mat < (diffuse_prob + mat_prob) {
                    // Metal
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_double_range(0.0, 0.5);
                    let material = Rc::new(Metal::new(albedo, fuzz));
                    world.add(Rc::new(Sphere::new(center, 0.2, material)));
                } else {
                    // Glass
                    let material = Rc::new(Dielectric::new(1.5));
                    world.add(Rc::new(Sphere::new(center, 0.2, material)));
                }
            }
        }
    }

    let material_1 = Rc::new(Dielectric::new(1.5));
    world.add(Rc::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, material_1)));

    let material_2 = Rc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Rc::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, material_2)));

    let material_3 = Rc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Rc::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, material_3)));

    (world, format!("weekend-{small_sphere_multiplier}-{}-{}", (diffuse_prob * 100.0).round() as u32, (mat_prob * 100.0).round() as u32))
}

#[allow(dead_code)]
pub fn simple_hollow_glass() -> (HittableList, String) {
    let mut world = HittableList::default();

    let material_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Rc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left = Rc::new(Dielectric::new(1.5));
    let material_bubble = Rc::new(Dielectric::new(1.0 / 1.5));
    let material_right = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    world.add(Rc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));
    world.add(Rc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5, material_center)));
    world.add(Rc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, material_left)));
    world.add(Rc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.4, material_bubble)));
    world.add(Rc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, material_right)));

    (world, "simple_hollow_glass".to_string())
}

#[allow(dead_code)]
pub fn simple_shiny_metal() -> (HittableList, String) {
    let mut world = HittableList::default();

    let material_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Rc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left = Rc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 1.0));
    let material_right = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    world.add(Rc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));
    world.add(Rc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5, material_center)));
    world.add(Rc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, material_left)));
    world.add(Rc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, material_right)));

    (world, "simple_shiny_metal".to_string())
}

#[allow(dead_code)]
pub fn simple_fuzzy_metal() -> (HittableList, String) {
    let mut world = HittableList::default();

    let material_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Rc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left = Rc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.3));
    let material_right = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    world.add(Rc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));
    world.add(Rc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5, material_center)));
    world.add(Rc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, material_left)));
    world.add(Rc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, material_right)));

    (world, "simple_fuzzy_metal".to_string())
}

#[allow(dead_code)]
pub fn simple_triangle() -> (HittableList, String) {
    let mut world = HittableList::default();
    let material_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_blue = Rc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_red = Rc::new(Lambertian::new(Color::new(0.9, 0.2, 0.2)));

    world.add(Rc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));
    let a = Point3::new(3.0, 0.0, -2.2);
    let b = Point3::new(-3.0, 0.0, -2.0);
    let c = Point3::new(1.0, 1.5, -1.9);
    world.add(Rc::new(Sphere::new(a, 0.1, material_red.clone())));
    world.add(Rc::new(Sphere::new(b, 0.1, material_red.clone())));
    world.add(Rc::new(Sphere::new(c, 0.1, material_red.clone())));
    world.add(Rc::new(Triangle::new(a, b, c, material_blue)));

    (world, "simple_triangle".to_string())
}

#[allow(dead_code)]
pub fn triangle_materials() -> (HittableList, String) {
    let mut world = HittableList::default();
    let material_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_red = Rc::new(Lambertian::new(Color::new(0.8, 0.2, 0.1)));
    let material_blue = Rc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_metal = Rc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 1.0));
    let material_glass = Rc::new(Dielectric::new(1.5));

    world.add(Rc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));

    let a = Point3::new(-1.0, 0.0, -2.2);
    let b = Point3::new(-3.0, 0.0, -2.0);
    let c = Point3::new(-2.0, 1.5, -1.9);
    world.add(Rc::new(Triangle::new(a, b, c, material_blue.clone())));

    let a = Point3::new(1.0, 0.0, -1.8);
    let b = Point3::new(-1.0, 0.0, -2.5);
    let c = Point3::new(0.0, 0.8, -1.0);
    world.add(Rc::new(Triangle::new(a, b, c, material_metal)));

    let a = Point3::new(3.0, 0.0, -1.8);
    let b = Point3::new(2.0, 0.0, -2.5);
    let c = Point3::new(1.0, 0.8, -1.0);
    world.add(Rc::new(Triangle::new(a, b, c, material_glass)));
    
    world.add(Rc::new(Sphere::new(Point3::new(1.0, 0.0, -1.5), 0.5, material_blue)));
    world.add(Rc::new(Sphere::new(Point3::new(1.8, 1.0, -2.0), 0.5, material_red)));

    (world, "triangle_materials".to_string())
}
