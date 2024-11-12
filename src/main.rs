use crate::color::Color;

mod vec3;
mod color;

fn main() {
    // Image
    let image_width = 256;
    let image_height = 256;

    let mut ppm_string = format!("P3\n{image_width} {image_height}\n255\n");

    for j in 0..image_height {
        eprintln!("Scanlines remaining: {}", image_height -j);
        for i in 0..image_width {
            let pixel_color = Color::new(
                i as f64 / (image_width - 1) as f64,
                j as f64 / (image_height - 1) as f64,
                0.0
            );
            ppm_string += &pixel_color.to_string();
        }
    }

    eprintln!("Done.");
    println!("{ppm_string}");
}
