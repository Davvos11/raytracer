use crate::color::Color;
use crate::data::Data;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;

pub fn ray_color_naive(r: &Ray, depth: u32, world: &dyn Hittable, data: &mut Data) -> Color {
    // Stop gathering light if the ray bounce limit is exceeded
    if depth == 0 {
        return Color::default();
    }

    let mut rec = HitRecord::default();

    if world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec, data) {
        let mut scattered = Ray::default();
        let mut attenuation = Color::default();
        if let Some(mat) = &rec.mat {
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                data.add_scatter_ray();
                return attenuation * ray_color_naive(&scattered, depth - 1, world, data);
            }
        }

        Color::default()
    } else {
        let unit_direction = r.direction().unit();
        let a = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}

