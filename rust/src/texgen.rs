use godot::classes::image::Format;
use godot::classes::{Image, ImageTexture};
use godot::prelude::*;

/// Builds a small procedural, tileable-ish texture: a base color with random speckles
/// and optional horizontal stripes. Simple enough to author entirely in code (no art
/// pipeline needed) while avoiding a flat, single-color "placeholder" look.
pub fn generate_texture(
    size: i32,
    base: Color,
    speckle: Color,
    speckle_chance: f64,
    stripe: Option<(Color, i32)>,
) -> Option<Gd<ImageTexture>> {
    let mut image = Image::create(size, size, false, Format::RGBA8)?;

    for y in 0..size {
        for x in 0..size {
            let mut c = base;

            if let Some((stripe_color, period)) = stripe {
                if period > 0 && y % period == 0 {
                    c = c.lerp(stripe_color, 0.5);
                }
            }

            if rand::random_bool(speckle_chance) {
                let t: f64 = rand::random_range(0.15..0.6);
                c = c.lerp(speckle, t);
            }

            image.set_pixel(x, y, c);
        }
    }

    ImageTexture::create_from_image(&image)
}
