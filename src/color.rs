use crate::interval::Interval;
use crate::vec3::Vec3;

pub type Color = Vec3;

pub fn color_to_string(pixel_color: &Color) -> String {
    let r = pixel_color.x();
    let g = pixel_color.y();
    let b = pixel_color.z();

    // Translate the [0,1] RGB values to the byte range [0,255]
    let intensity = Interval::new(0.0, 0.999);
    let rbyte = (256.0 * intensity.clamp(r)) as u32;
    let gbyte = (256.0 * intensity.clamp(g)) as u32;
    let bbyte = (256.0 * intensity.clamp(b)) as u32;

    format!("{rbyte} {gbyte} {bbyte}\n")
}