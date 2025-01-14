use crate::color::{color_to_string, Color};
use crate::data::Data;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::{degrees_to_radians, random_double};
use crate::vec3::{Point3, Vec3};
use indicatif::{ProgressBar, ProgressStyle};
use std::io;
use std::io::Write;

#[derive(Default)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub vfov: f64,
    pub look_from: Point3,
    pub look_at: Point3,
    pub v_up: Vec3,
    pub defocus_angle: f64,
    pub focus_dist: f64,
    image_height: u32,
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Point3,
    pixel_delta_v: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            look_at: Point3::new(0.0, 0.0, -1.0),
            v_up: Vec3::new(0.0, 1.0, 0.0),
            focus_dist: 10.0,
            ..Default::default()
        }
    }

    pub fn render(&mut self, world: &HittableList, writer: &mut impl Write, data: &mut Data) -> io::Result<()> {
        self.initialise();

        // Display progress bar
        let bar = ProgressBar::new(self.image_height as u64);
        bar.set_style(ProgressStyle::default_bar()
            .template("{wide_bar} Lines completed: {percent}%, Elapsed: {elapsed_precise}, ETA: {eta_precise}").unwrap());

        let header = format!("P3\n{} {}\n255\n", self.image_width /*- 180*/, self.image_height /*- 110*/);
        writer.write_all(header.as_bytes())?;

        for j in /*11*/0..self.image_height {
            bar.inc(1);

            for i in /*18*/0..self.image_width {
                let mut pixel_color = Color::default();
                for _ in 0..self.samples_per_pixel {
                    data.add_primary_ray();
                    let r = self.get_ray(i, j);
                    pixel_color += ray_color(&r, self.max_depth, world, data);
                }

                let pixel = color_to_string(&(self.pixel_samples_scale * pixel_color));
                writer.write_all(pixel.as_bytes())?;
                // return Ok(())
            }
        }

        Ok(())
    }

    pub fn image_height(&self) -> u32 {
        (self.image_width as f64 / self.aspect_ratio) as u32
    }
    
    fn initialise(&mut self) {
        // Image setup
        self.image_height = self.image_height();
        self.image_height = if self.image_height < 1 { 1 } else { self.image_height };

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.look_from;

        // Determine viewport
        let theta = degrees_to_radians(self.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // Calculate u,v,w unit basis vectors for the camera coordinate frame
        self.w = (self.look_from - self.look_at).unit();
        self.u = self.v_up.cross(&self.w).unit();
        self.v = self.w.cross(&self.u);

        // Calculate vectors across viewport edges
        let viewport_u = viewport_width * self.u;
        let viewport_v = viewport_height * -self.v;

        // Calculate delta vectors from pixel to pixel
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // Calculate location of upper left pixel
        let viewport_upper_left = self.center
            - (self.focus_dist * self.w)
            - (viewport_u / 2.0)
            - (viewport_v / 2.0);
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        // Calculate camera defocus disk basis vectors
        let defocus_radius =
            self.focus_dist * degrees_to_radians(self.defocus_angle / 2.0).tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    /// Construct a camera ray originating from the defocus disk and directed
    /// at randomly sampled point around the pixel location i, j.
    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = sample_square();
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x()) * self.pixel_delta_u)
            + ((j as f64 + offset.y()) * self.pixel_delta_v);

        let ray_origin =
            if self.defocus_angle <= 0.0 { self.center } else { self.defocus_disk_sample() };
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = Point3::random_in_unit_disk();
        self.center + (p.x() * self.defocus_disk_u) + (p.y() * self.defocus_disk_v)
    }
}


/// Determine the ray colour for the ray tracing algorithm
fn ray_color(r: &Ray, depth: u32, world: &dyn Hittable, data: &mut Data) -> Color {
    // Stop gathering light if the ray bounce limit is exceeded
    if depth == 0 {
        return Color::default();
    }

    let mut rec = HitRecord::default();

    if world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec, data) {
        if rec.hits_aabb_edge {
            return Color::red();
        }

        let mut scattered = Ray::default();
        let mut attenuation = Color::default();

        if let Some(mat) = &rec.mat {
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                data.add_scatter_ray();
                return attenuation * ray_color(&scattered, depth - 1, world, data);
            }
        }

        Color::default()
    } else {
        if rec.hits_aabb_edge {
            return Color::new(1.0, 0.0, 0.0);
        }

        let unit_direction = r.direction().unit();
        let a = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}

/// Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
fn sample_square() -> Vec3 {
    Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
}

