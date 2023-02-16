use image::{Rgb, RgbImage};

fn main() {
    let mut img = RgbImage::new(1920, 1080);
    let w = img.width() as f32;
    let h = img.height() as f32;
    for (x, y, p) in img.enumerate_pixels_mut() {
        *p = Rgb([
            (255f32 / w * x as f32) as u8,
            (255f32 / h * y as f32) as u8,
            (255f32 * (x as f32 + y as f32)/(w + h)) as u8,
        ])
    }
    img.save("img.png").unwrap()
}
