use crate::color::{color_to_string, Color};
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

#[derive(Default)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    image_height: u32,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Point3,
    pixel_delta_v: Point3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            ..Default::default()
        }
    }
    
    pub fn render(&mut self, world: &dyn Hittable) {
        self.initialise();

        let mut ppm_string = format!("P3\n{} {}\n255\n", self.image_width, self.image_height);

        for j in 0..self.image_height {
            eprintln!("Scanlines remaining: {}", self.image_height - j);
            for i in 0..self.image_width {
                let pixel_center = self.pixel00_loc +
                    (i as f64 * self.pixel_delta_u) + (j as f64 * self.pixel_delta_v);
                let ray_direction = pixel_center - self.center;
                let r = Ray::new(self.center, ray_direction);

                let pixel_color = Self::ray_color(&r, world);
                ppm_string += color_to_string(&pixel_color).as_str();
            }
        }

        eprintln!("Done.");
        println!("{ppm_string}");
    }

    fn initialise(&mut self) {
        // Image setup
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as u32;
        self.image_height = if self.image_height < 1 { 1 } else { self.image_height };

        self.center = Point3::default();

        // Determine viewport
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // Calculate vectors across viewport edges
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Calculate delta vectors from pixel to pixel
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // Calculate location of upper left pixel
        let viewport_upper_left = self.center
            - Vec3::new(0.0, 0.0, focal_length)
            - (viewport_u / 2.0)
            - (viewport_v / 2.0);
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);
    }

    fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
        let mut rec = HitRecord::default();
        if world.hit(r, Interval::new(0.0, f64::INFINITY), &mut rec) {
            0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0))
        } else {
            let unit_direction = r.direction().unit();
            let a = 0.5 * (unit_direction.y() + 1.0);
            (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
        }
    }
}