use image::{Rgb, RgbImage};
mod vector3;

fn main() {
    let mut img = RgbImage::new(1920, 1080);
    let w = img.width() as f32;
    let h = img.height() as f32;
    for (x, y, p) in img.enumerate_pixels_mut() {
        *p = Rgb([
            (255.999 / w * x as f32) as u8,
            (255.999 / h * y as f32) as u8,
            (255.999 * x as f32/h) as u8,
        ])
    }
    img.save("img.png").unwrap()
}
