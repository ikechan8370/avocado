#[cfg(test)]
mod tests {
    use std::path::Path;
    use ab_glyph::{FontRef, PxScale};
    use image::{GenericImageView, Rgb, Rgba, RgbaImage, RgbImage};
    use image::imageops::FilterType;
    use imageproc::drawing::{draw_cross_mut, draw_filled_rect_mut, draw_text_mut, text_size};
    use imageproc::rect::Rect;

    #[test]
    fn it_works() {
        let path = Path::new("out.png");
        let width = 720;
        let height = 1800;
        let mut image = RgbaImage::new(width, height);
        let green = Rgba([255u8, 255u8, 255u8, 255u8]);
        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(width, height), green);

        let height = 64.;
        let scale = PxScale {
            x: height,
            y: height,
        };


        let overlay_img = image::open("logo.png").unwrap().to_rgba8();
        let overlay_img = image::imageops::resize(&overlay_img, height as u32, height as u32, FilterType::Lanczos3);
        let (overlay_width, overlay_height) = overlay_img.dimensions();

        let font = FontRef::try_from_slice(include_bytes!("..\\..\\SmileySans-Oblique.ttf")).unwrap();

        let text = "Avocado Bot";
        let (w, h) = text_size(scale, &font, text);

        let x = (width - w - height as u32) / 2;
        let y = 30;
        for j in 0..overlay_height {
            for i in 0..overlay_width {
                let pixel = overlay_img.get_pixel(i, j);
                if pixel[3] != 0 { // 检查Alpha通道，只处理非完全透明的像素
                    let base_pixel = image.get_pixel(i + x, j + y);
                    let blended_pixel = blend(base_pixel, &pixel);
                    image.put_pixel(i + x, j + y, blended_pixel);
                }
            }
        }

        draw_text_mut(&mut image, Rgba([0u8, 0u8, 255u8, 255u8]), ((width - w - height as u32) / 2 + height as u32) as i32, 30, scale, &font, text);

        image.save(path).unwrap();
    }

    // 简单的Alpha混合实现
    fn blend(under: &Rgba<u8>, over: &Rgba<u8>) -> Rgba<u8> {
        let alpha_over = over[3] as f32 / 255.0;
        let alpha_under = under[3] as f32 / 255.0;
        let alpha_out = alpha_over + alpha_under * (1.0 - alpha_over);

        let blend_channel = |under: u8, over: u8| -> u8 {
            ((over as f32 * alpha_over + under as f32 * alpha_under * (1.0 - alpha_over)) / alpha_out).round() as u8
        };

        Rgba([
            blend_channel(under[0], over[0]),
            blend_channel(under[1], over[1]),
            blend_channel(under[2], over[2]),
            (alpha_out * 255.0).round() as u8,
        ])
    }
}
