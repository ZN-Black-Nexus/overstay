use godot::classes::image::Format;
use godot::classes::{Image, ImageTexture};
use godot::prelude::*;

/// Builds a small procedural, tileable-ish texture: a base color with soft blob
/// stains (water/dirt damage, not per-pixel static), an optional horizontal
/// stripe pattern (e.g. ceiling tile grid), and an optional vertical seam
/// pattern (wallpaper roll seams). Simple enough to author entirely in code
/// (no art pipeline needed) while avoiding a flat, single-color look.
pub fn generate_texture(
    size: i32,
    base: Color,
    stain: Color,
    stain_count: i32,
    horizontal_stripe: Option<(Color, i32)>,
    vertical_seam: Option<(Color, i32)>,
) -> Option<Gd<ImageTexture>> {
    let mut image = Image::create(size, size, false, Format::RGBA8)?;

    for y in 0..size {
        for x in 0..size {
            let mut c = base;
            if let Some((stripe_color, period)) = horizontal_stripe {
                if period > 0 && y % period == 0 {
                    c = c.lerp(stripe_color, 0.5);
                }
            }
            if let Some((seam_color, period)) = vertical_seam {
                if period > 0 && x % period == 0 {
                    c = c.lerp(seam_color, 0.4);
                }
            }
            image.set_pixel(x, y, c);
        }
    }

    for _ in 0..stain_count {
        let cx = rand::random_range(0..size);
        let cy = rand::random_range(0..size);
        let radius: f32 = rand::random_range((size / 8) as f32..(size / 3) as f32);
        let strength: f64 = rand::random_range(0.2..0.5);

        let min_x = (cx as f32 - radius).max(0.0) as i32;
        let max_x = (cx as f32 + radius).min(size as f32 - 1.0) as i32;
        let min_y = (cy as f32 - radius).max(0.0) as i32;
        let max_y = (cy as f32 + radius).min(size as f32 - 1.0) as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 - cx as f32;
                let dy = y as f32 - cy as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= radius {
                    let falloff = (1.0 - dist / radius) as f64;
                    let existing = image.get_pixel(x, y);
                    image.set_pixel(x, y, existing.lerp(stain, strength * falloff));
                }
            }
        }
    }

    ImageTexture::create_from_image(&image)
}
