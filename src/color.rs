use crate::vec3::Vec3;

pub type Color = Vec3;

pub fn color_to_string(pixel_color: &Color) -> String {
    let r = pixel_color.x();
    let g = pixel_color.y();
    let b = pixel_color.z();

    let ir= (255.999 * r) as i32;
    let ig= (255.999 * g) as i32;
    let ib= (255.999 * b) as i32;

    format!("{ir} {ig} {ib}\n")
}