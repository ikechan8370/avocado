#[cfg(test)]
mod tests {
    use std::path::Path;
    use ab_glyph::{FontRef, PxScale};
    use image::{GenericImageView, ImageFormat, Rgb, Rgba, RgbaImage, RgbImage};
    use image::imageops::FilterType;
    use imageproc::drawing::{draw_cross_mut, draw_filled_rect_mut, draw_text_mut, text_size};
    use imageproc::rect::Rect;
    use crate::utils::image::{DEFAULT_NORMAL_FONT, is_emoji, overlay_image, overlay_image_from_url, overlay_image_with_pure_color, OverlayImageOption, render_text_with_different_fonts};
    #[tokio::test]
    async fn emoji_works() {
        assert_eq!(true, is_emoji('🥑'));
        assert_eq!(false, is_emoji('好'));
        assert_eq!(false, is_emoji('e'));
        assert_eq!(false, is_emoji('1'));
    }


    #[tokio::test]
    async fn it_works() {
        let path = Path::new("out.png");
        let width = 720;
        let height = 1800;
        let mut image = RgbaImage::new(width, height);
        let green = Rgba([255u8, 255u8, 255u8, 255u8]);
        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(width, height), green);


        // draw title
        let height = 64.;
        let scale = PxScale {
            x: height,
            y: height,
        };


        let overlay_img = image::open("logo.png").unwrap().to_rgba8();
        let overlay_img = image::imageops::resize(&overlay_img, height as u32, height as u32, FilterType::Lanczos3);

        let font = DEFAULT_NORMAL_FONT.clone();

        let text = "Avocado Bot";
        let (w, h) = text_size(scale, &font, text);

        let x = (width - w - height as u32) / 2;
        let y = 30;
        let theme_color = Rgba([84u8, 128u8, 2u8, 255u8]);
        overlay_image((x, y), &mut image, &overlay_img, OverlayImageOption::new()).unwrap();

        draw_text_mut(&mut image, theme_color, ((width - w - height as u32) / 2 + height as u32) as i32, 30, scale, &font, text);

        // icon
        let x = 50;
        let y = 30 + h + 40;
        let crop_height = width / 2;
        overlay_image_from_url((x, y), &mut image, "https://q1.qlogo.cn/g?b=qq&nk=450960006&s=0".to_string(),
                               OverlayImageOption::new()
                                   .resize(width / 2, width / 2)
                                   .crop(crop_height / 6, 0, crop_height * 2 / 3, crop_height)
        ).await.unwrap();
        let icon_right_x = x + crop_height * 2 / 3;
        // text on the right side of icon
        let black = Rgba([0u8, 0u8, 0u8, 255u8]);
        let scale = PxScale {
            x: 30.,
            y: 30.,
        };
        let text = "墨西哥鳄梨酱🥑(450960006)";
        // draw_text_mut(&mut image, black, (icon_right_x + 20) as i32, y as i32, scale, &font, text);
        render_text_with_different_fonts(&mut image, text.to_string(), ((icon_right_x + 20) as i32, y as i32), None, scale, black).unwrap();

        // draw_text_mut(&mut image, theme_color, ((width - w - height as u32) / 2 + height as u32) as i32, 30, scale, &font, text);


        image.save(path).unwrap();
    }
}
