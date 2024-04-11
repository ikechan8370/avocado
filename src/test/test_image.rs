#[cfg(test)]
mod tests {
    use std::cmp::max;
    use std::path::Path;
    use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
    use image::{GenericImageView, ImageFormat, Rgb, Rgba, RgbaImage, RgbImage};
    use image::imageops::FilterType;
    use imageproc::drawing::{draw_cross_mut, draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment, draw_line_segment_mut, draw_text_mut, text_size};
    use imageproc::rect::Rect;
    use sysinfo::{Disks, System};
    use unicode_segmentation::UnicodeSegmentation;
    use crate::utils::image::{DEFAULT_NORMAL_FONT, draw_filled_rect_with_circle_corner, is_emoji, overlay_image, overlay_image_from_url, overlay_image_with_pure_color, OverlayImageOption, render_text_with_different_fonts};
    #[tokio::test]
    async fn emoji_works() {
        assert_eq!(true, emojis::get("❤️‍🔥").is_some());
        assert_eq!(true, is_emoji("🥑".to_string()).await);
        assert_eq!(false, is_emoji("好".to_string()).await);
        assert_eq!(false, is_emoji("好1".to_string()).await);
        assert_eq!(true, is_emoji("❤️‍🔥".to_string()).await);
    }

    #[test]
    fn emoji_segment() {
        use unicode_segmentation::UnicodeSegmentation;
        let s = "墨西哥鳄梨酱🥑❤️‍🔥";
        let w = s.split_word_bounds().collect::<Vec<&str>>();
        let b: &[_] = &["墨", "西", "哥", "鳄", "梨", "酱", "🥑", "❤️‍🔥"];
        println!("{:?}", w);
        assert_eq!(w, b);
    }

    #[tokio::test]
    async fn it_works() {
        let path = Path::new("out.png");
        let width = 1080;
        let height = 2240;
        let mut image = RgbaImage::new(width, height);
        let green = Rgba([255u8, 255u8, 255u8, 255u8]);
        let white = Rgba([255u8, 255u8, 255u8, 255u8]);
        let grey = Rgba([150u8, 150u8, 150u8, 255u8]);
        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(width, height), green);
        let padding_left_right = 75;

        // draw title
        let height = 96.;
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

        // draw_text_mut(&mut image, theme_color, ((width - w - height as u32) / 2 + height as u32) as i32, 30, scale, &font, text);
        draw_bold_weight(&mut image, 30, (width - w - height as u32) / 2 + height as u32, theme_color, scale, &font, text);
        // icon
        let x = padding_left_right;
        let y = 30 + h + 50;
        let crop_height = width / 3;
        overlay_image_from_url((x, y), &mut image, "https://q1.qlogo.cn/g?b=qq&nk=450960006&s=0".to_string(),
                               OverlayImageOption::new()
                                   .resize(width / 3, width / 3)
                                   .crop(crop_height / 6, 0, crop_height * 2 / 3, crop_height)
        ).await.unwrap();
        let icon_right_x = x + crop_height * 2 / 3 + 30;
        let _ = draw_line_segment_mut(&mut image, (icon_right_x as f32, y as f32 + 30.), (icon_right_x as f32, (y + crop_height - 10) as f32), grey);

        // text on the right side of icon
        let black = Rgba([0u8, 0u8, 0u8, 255u8]);
        let text_left_x = (icon_right_x + 40) as i32;
        let mut current_y = y as i32 + 30;
        let font_size = 56.;
        let scale = PxScale {
            x: font_size,
            y: font_size,
        };
        let text = "墨西哥鳄梨酱🥑";
        // draw_text_mut(&mut image, black, (icon_right_x + 20) as i32, y as i32, scale, &font, text);
        render_text_with_different_fonts(&mut image, Rgba([88, 214, 141, 255]), text_left_x, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += font_size as i32;



        let font_size = 40;
        let scale = PxScale {
            x: font_size as f32,
            y: font_size as f32,
        };
        let line_height = 10;
        current_y += line_height * 2;

        let text = "💻 本次在线：7天12小时37分58秒";
        render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += font_size + line_height;

        let text = "👨‍👩‍👧‍👧 群聊：12";
        render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += font_size + line_height;

        let text = "🤵‍♂️ 好友：8";
        render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += font_size + line_height;

        let text = "❤️‍🔥 Powered by OpenShamrock-1.0.1-59d762e";
        render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += font_size + line_height;

        let text = "📱 QQ v9.0.17 on Android Pad";
        render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += font_size + line_height;

        // split line
        current_y = max((y + crop_height) as i32, current_y);
        current_y += 40;

        let _ = draw_line_segment_mut(&mut image, (padding_left_right as f32, current_y as f32), ((width as i32 - 50) as f32, current_y as f32), grey);
        current_y += 32;

        // draw system stats
        let font_size = 64;
        let bar_height = 48;
        let bar_radius = 10;

        let scale = PxScale {
            x: font_size as f32,
            y: font_size as f32,
        };
        let estimated_title_width = 130;
        let estimated_title_height = 64;
        let bar_bg_color = Rgba([200u8, 200u8, 200u8, 255u8]);
        let chart_start_x = padding_left_right + estimated_title_width;
        let bar_width = width - chart_start_x - padding_left_right;
        let blue = Rgba([0u8, 0u8, 255u8, 255u8]);
        let content_distant = 4;
        let content_height = (bar_height - content_distant * 2) as u32;


        let content_font_size = 36;
        let content_scale = PxScale {
            x: content_font_size as f32,
            y: content_font_size as f32,
        };
        let text = "CPU: ";
        draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
        draw_bar(&mut image, current_y, "50%", 0.5, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_font_size as f32, content_scale, white);
        current_y += font_size + 20;

        let text = "内存: ";
        draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
        draw_bar(&mut image, current_y, "98%", 0.98, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_font_size as f32, content_scale, white);

        current_y += font_size + 20;

        let text = "交换: ";
        draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
        draw_bar(&mut image, current_y, "99%", 0.99, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_font_size as f32, content_scale, white);

        current_y += font_size + 20;

        let text = "硬盘: ";
        draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
        draw_bar(&mut image, current_y, "48%", 0.48, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_font_size as f32, content_scale, white);

        current_y += font_size + 20;

        let text = "网络: ";
        let half_width = (width - padding_left_right * 2) / 2;
        let half_content_width = half_width - estimated_title_width;
        draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);

        // draw_bar(&mut image, current_y, "98%", 0.98, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, blue, content_distant, content_height, content_font_size as f32, content_scale, white);
        let up_text = "⬆ 1.2MB/s";
        let down_text = "⬇ 2.3MB/s";
        let network_font_size = 42;
        let network_scale = PxScale {
            x: network_font_size as f32,
            y: network_font_size as f32,
        };
        let net_start_x = padding_left_right + estimated_title_width;
        let network_text_top = current_y  + (estimated_title_height - network_font_size) / 2;
        render_text_with_different_fonts(&mut image, black, net_start_x as i32, network_text_top, network_scale, up_text.to_string(), None).await.unwrap();
        render_text_with_different_fonts(&mut image, black, net_start_x as i32 + (half_content_width / 2) as i32, network_text_top, network_scale, down_text.to_string(), None).await.unwrap();

        let text = "在线: ";
        let uptime_left = padding_left_right + half_width;
        let uptime_content_left = uptime_left + estimated_title_width;
        draw_bold_weight(&mut image, current_y, uptime_left, black, scale, &font, text);
        let uptime_font_size = 48;
        let uptime_scale = PxScale {
            x: uptime_font_size as f32,
            y: uptime_font_size as f32,
        };
        let uptime_text_top = current_y  + (estimated_title_height - uptime_font_size) / 2;
        let uptime_text = "7天12小时37分58秒";
        let (content_text_width, content_text_height) = get_text_size(&font, uptime_text, uptime_scale);
        let uptime_text_left = uptime_content_left + (half_content_width - content_text_width as u32) / 2;
        let uptime_text_top =  current_y  + (estimated_title_height - content_text_height as i32) / 2;
        render_text_with_different_fonts(&mut image, black, uptime_text_left as i32, uptime_text_top, uptime_scale, uptime_text.to_string(), None).await.unwrap();

        current_y += font_size + 30;

        let _ = draw_line_segment_mut(&mut image, (padding_left_right as f32, current_y as f32), ((width - padding_left_right) as f32, current_y as f32), grey);

        current_y += 30;

        let brand_font_size = 48;
        let scale = PxScale {
            x: brand_font_size as f32,
            y: brand_font_size as f32,
        };

        // 88, 214, 141
        let color = Rgba([0, 149, 26, 255]);

        let text = "🖥  操作系统 —— Ubuntu 20.04.2 LTS";
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;

        let text = "🏅  主机名称 —— avocadobot";
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;

        let text = "🧿  内核 —— 5.4.0-74-generic";
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;

        let text = "💾  CPU —— Intel(R) Core(TM) i7-7700HQ CPU @ 2.80GHz * 8";
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;

        let text = "🧠  内存 —— 16GB";
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;

        let text = "🪧  进程总数 —— 123";
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;

        let text = "🥑  本进程占用 —— CPU 12.3% 内存 1.2GB";
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;

        current_y += 20;
        let _ = draw_line_segment_mut(&mut image, (padding_left_right as f32, current_y as f32), ((width - padding_left_right) as f32, current_y as f32), grey);
        current_y += 20;

        let foot_font_size = 32;
        let foot_scale = PxScale {
            x: foot_font_size as f32,
            y: foot_font_size as f32,
        };
        let text = "Created by project \u{E045} https://github.com/ikechan8370/avocado";
        let (foot_width, _) = get_text_size(&font, text, foot_scale);
        let foot_start = padding_left_right + (width - 2 * padding_left_right - foot_width as u32) / 2;
        render_text_with_different_fonts(&mut image, grey, foot_start as i32, current_y, foot_scale, text.to_string(), None).await.unwrap();
        current_y += foot_font_size;

        image = image::imageops::crop(&mut image, 0, 0, width, current_y as u32 + 30).to_image();
        image.save(path).unwrap();
    }

    fn draw_bar(image: &mut RgbaImage, current_y: i32, content_text: &str, rate: f64, font: &FontRef, font_size: i32, bar_height: i32, bar_bg_color: Rgba<u8>, bar_radius: u32, chart_start_x: u32, bar_width: u32, content_distant: i32, content_height: u32, content_font_size: f32, content_scale: PxScale, text_color: Rgba<u8>) {
        draw_filled_rect_with_circle_corner(image, Rect::at(chart_start_x as i32, current_y + (font_size - bar_height) / 2).of_size(bar_width, bar_height as u32), bar_bg_color, bar_radius);
        let content_length = ((bar_width as i32 - 2 * content_distant) as f64 * rate) as u32;
        let content_left = chart_start_x as i32 + content_distant;
        let content_top = current_y + (font_size - bar_height) / 2 + content_distant;
        let content_color = match rate {
            r if r < 0.5 => Rgba([88u8, 214u8, 141u8, 255u8]),
            r if r < 0.8 => Rgba([70u8, 173u8, 255u8, 255u8]),
            _ => Rgba([199, 0, 57, 255u8]),
        };
        draw_filled_rect_with_circle_corner(image, Rect::at(content_left, content_top).of_size(content_length, content_height), content_color, bar_radius - (content_distant / 2) as u32);
        let (content_text_width, content_text_height) = get_text_size(font, content_text, content_scale);
        let content_text_top = content_top + ((content_height as f32 - content_text_height) / 2.) as i32;
        let content_tet_left = content_left + ((content_length as f32 - content_text_width) / 2.) as i32;
        draw_text_mut(image, text_color, content_tet_left, content_text_top, content_scale, &font, content_text);
    }

    fn get_text_size(font: &FontRef, text: &str, scale: PxScale) -> (f32, f32) {
        let mut content_text_width = 0.;
        let mut content_text_height: f32 = 0.;
        for x in text.chars() {
            let scaled_font = font.as_scaled(scale);
            let glyph = scaled_font.scaled_glyph(x);
            let h_metrics = scaled_font.h_advance(glyph.id);
            content_text_width += h_metrics;
            content_text_height = content_text_height.max(scaled_font.height());
        }
        (content_text_width, content_text_height)
    }

    fn draw_bold_weight(image: &mut RgbaImage, current_y: i32, padding_left_right: u32, black: Rgba<u8>, scale: PxScale, font: &FontRef, text: &str) {
        for offset_x in &[0.0, 1.0, -1.0] {
            for offset_y in &[0.0, 1.0, -1.0]  {
                draw_text_mut(image, black, (padding_left_right as f64 + offset_x.clone()) as i32, (current_y as f64 + offset_y.clone()) as i32, scale, &font, text);
            }
        }
    }

    #[tokio::test]
    async fn overlay_image_works() {
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            println!("{disk:?}");
        }
    }

    #[tokio::test]
    async fn cpu() {
        let mut system = System::new();
        system.refresh_all();
        let cpu = system.global_cpu_info();
        println!("{}", cpu.cpu_usage());
        println!("{}", cpu.name());
        println!("{}", cpu.vendor_id());
        println!("{}", cpu.brand());
        println!("{}", cpu.frequency());
        for cpu in system.cpus() {
            println!("{}", cpu.cpu_usage());
            println!("{}", cpu.name());
            println!("{}", cpu.vendor_id());
            println!("{}", cpu.brand());
            println!("{}", cpu.frequency());
        }
    }
}
