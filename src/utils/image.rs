use std::fs::File;
use std::future::Future;
use std::io::{BufReader, Cursor, Read};
use std::pin::Pin;
use std::sync::Arc;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use dashmap::DashMap;
use image::{Rgba, RgbaImage};
use image::imageops::FilterType;
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use lazy_static::lazy_static;
use log::{debug, error};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use unicode_segmentation::UnicodeSegmentation;
use crate::model::error::Result;
use zip::ZipArchive;
use crate::err;

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

pub fn overlay_image(position: (u32, u32), image: &mut RgbaImage, overlay_img: &RgbaImage, option: OverlayImageOption) -> Result<()> {

    let mut overlay_img = overlay_img.clone();

    let (mut overlay_width, mut overlay_height) = overlay_img.dimensions();

    option.need_resize.then(|| {
        if let Some(resize_x) = option.resize_x {
            overlay_width = resize_x;
        }
        if let Some(resize_y) = option.resize_y {
            overlay_height = resize_y;
        }
        overlay_img = image::imageops::resize(&overlay_img, overlay_width, overlay_height, option.filter_type);
    });

    option.need_crop.then(|| {
        if let Some(crop_start_x) = option.crop_start_x {
            if let Some(crop_start_y) = option.crop_start_y {
                if let Some(crop_width) = option.crop_width {
                    if let Some(crop_height) = option.crop_height {
                        let (width, height) = overlay_img.dimensions();
                        if crop_start_x + crop_width > width {
                            overlay_width = width - crop_start_x;
                        }
                        if crop_start_y + crop_height > height {
                            overlay_height = height - crop_start_y;
                        }
                        overlay_img = image::imageops::crop(&mut overlay_img, crop_start_x, crop_start_y, crop_width, crop_height).to_image();
                    }
                }
            }
        }
    });

    let (x, y) = position;
    let (overlay_width, overlay_height) = overlay_img.dimensions();
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
    Ok(())
}

#[derive(Debug, Clone)]
pub struct OverlayImageOption {
    pub resize_x: Option<u32>,
    pub resize_y: Option<u32>,
    pub filter_type: FilterType,
    pub format: Option<image::ImageFormat>,
    pub crop_start_x: Option<u32>,
    pub crop_start_y: Option<u32>,
    pub crop_width: Option<u32>,
    pub crop_height: Option<u32>,
    need_resize: bool,
    need_crop: bool
}

impl OverlayImageOption {
    pub fn new() -> Self {
        Self {
            resize_x: None,
            resize_y: None,
            filter_type: FilterType::Lanczos3,
            format: None,
            crop_start_x: None,
            crop_start_y: None,
            crop_width: None,
            crop_height: None,
            need_resize: false,
            need_crop: false,
        }
    }

    pub fn resize(mut self, x: u32, y: u32) -> Self {
        self.resize_x = Some(x);
        self.resize_y = Some(y);
        self.need_resize = true;
        self
    }

    pub fn crop(mut self, start_x: u32, start_y: u32, width: u32, height: u32) -> Self {
        self.crop_start_x = Some(start_x);
        self.crop_start_y = Some(start_y);
        self.crop_width = Some(width);
        self.crop_height = Some(height);
        self.need_crop = true;
        self
    }

    pub fn filter_type(mut self, filter_type: FilterType) -> Self {
        self.filter_type = filter_type;
        self
    }

    pub fn format(mut self, format: image::ImageFormat) -> Self {
        self.format = Some(format);
        self
    }
}

pub async fn image_from_url(url: String) -> Result<RgbaImage> {
    let response = reqwest::get(url.clone()).await?;

    // 将响应的主体部分作为字节流读取。
    let bytes = response.bytes().await?;

    // 使用`image`库加载和转换图片。
    let format = image::ImageFormat::from_path(url.as_str()).unwrap_or(image::guess_format(&bytes)?);

    // 使用Cursor将字节流转换为一个`Read`实例，因为`image::load`需要一个实现了`Read`的输入。
    let cursor = Cursor::new(bytes);

    let img = image::load(cursor, format)?.to_rgba8();
    Ok(img)
}

pub async fn overlay_image_from_url(position: (u32, u32), image: &mut RgbaImage, overlay_img_url: String, option: OverlayImageOption) -> Result<()> {
    let overlay_img = image_from_url(overlay_img_url).await?;
    overlay_image(position, image, &overlay_img, option)
}

pub fn overlay_image_with_pure_color(position: (u32, u32), image: &mut RgbaImage, overlay_img: &RgbaImage, color: Rgba<u8>) {
    let (x, y) = position;
    let (overlay_width, overlay_height) = overlay_img.dimensions();
    for j in 0..overlay_height {
        for i in 0..overlay_width {
            let pixel = overlay_img.get_pixel(i, j);
            if pixel[3] != 0 { // 检查Alpha通道，只处理非完全透明的像素
                let base_pixel = image.get_pixel(i + x, j + y);
                let blended_pixel = blend(base_pixel, &color);
                image.put_pixel(i + x, j + y, blended_pixel);
            }
        }
    }
}
type FontFn<'font> = fn(String) -> Pin<Box<dyn Future<Output = FontRef<'font>> + Send + 'font>>;

lazy_static!(
    // pub static ref DEFAULT_EMOJI_FONT: FontRef<'static> = FontRef::try_from_slice(include_bytes!("../../resources/font/NotoColorEmoji.ttf")).unwrap();
    pub static ref DEFAULT_NORMAL_FONT: FontRef<'static> = FontRef::try_from_slice(include_bytes!("../../resources/font/SmileySans-Oblique.ttf")).unwrap();
);
pub async fn select_font_for_char<'font>(_c: String) -> FontRef<'font> {
    DEFAULT_NORMAL_FONT.clone()
}

pub static EMOJI_MAP: Lazy<Arc<RwLock<DashMap<String, bool>>>> = Lazy::new(|| {
    let emoji_map = DashMap::new();
    Arc::new(RwLock::new(emoji_map))
});

// 用于判断一个字符是否是emoji
pub async fn is_emoji(c: String) -> bool {
    let exist = {
        let map = EMOJI_MAP.read().await;
        map.get(&c).map(|value_ref| value_ref.clone())
    };
    if let Some(exist) = exist {
        return exist;
    }
    if c.is_ascii() {
        return false;
    }
    // c 是像 \u{E045}这样的字符
    let is = emojis::get(c.as_str()).is_some() || {
        let file = File::open("resources/font/openmoji-72x72-color.zip").unwrap();
        let reader = BufReader::new(file);
        let mut zip = ZipArchive::new(reader).unwrap();
        let emoji = emoji_to_unicode_string(c.as_str()).to_uppercase();
        let filename = format!("{}.png", emoji);
        let found = zip.by_name(filename.as_str());
        found.is_ok()
    };
    if is {
        let map = EMOJI_MAP.write().await;
        map.insert(c.clone(), is);
    }
    is
}

fn emoji_to_unicode_string(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_ascii() {
                // 跳过ASCII字符，因为我们只关注Unicode扩展字符
                None
            } else {
                // 将非ASCII字符（Unicode扩展字符）转换为十六进制形式
                Some(format!("{:X}", c as u32))
            }
        })
        .collect::<Vec<String>>() // 将转换结果收集到Vec<String>中
        .join("-") // 使用"-"连接各部分
}

pub fn get_emoji_png(emoji_name: &str) -> Result<RgbaImage> {
    // 打开ZIP文件
    let file = File::open("resources/font/openmoji-72x72-color.zip").unwrap();
    let reader = BufReader::new(file);
    let mut zip = ZipArchive::new(reader)?;

    // 尝试从ZIP中找到指定的PNG文件
    let emoji = emoji_to_unicode_string(emoji_name).to_uppercase();
    let filename = format!("{}.png", emoji);
    debug!("Trying to find file: {}", filename);
    let png_file = zip.by_name(filename.as_str())?;
    let bytes: Vec<u8> = png_file.bytes().map(|b| b.unwrap()).collect();
    // 使用`image`库加载PNG文件
    let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?;

    Ok(image.to_rgba8())

}
fn wrapper<'a>(name: String) -> Pin<Box<dyn Future<Output = FontRef<'a>> + Send + 'a>> {
    Box::pin(select_font_for_char(name))
}

pub async fn render_text_with_different_fonts<'a>(image: &mut RgbaImage, color: Rgba<u8>, x: i32, y: i32, font_scale: PxScale, text: String, font_map: Option<FontFn<'a>>) -> Result<()> {
    let chars = text.as_str().split_word_bounds().map(String::from).collect::<Vec<String>>();
    let font_map = font_map.unwrap_or(wrapper);
    let mut cursor_x = x as f32;
    for c in chars {
        if is_emoji(c.clone()).await {
            get_emoji_png(c.as_str()).and_then(|emoji_img| {
                overlay_image((cursor_x as u32, y as u32), image, &emoji_img, OverlayImageOption::new().resize(font_scale.x as u32, font_scale.y as u32))?;
                cursor_x += font_scale.x;
                Ok(())
            }).or_else(|e| {
                error!("Failed to render emoji: {}", e);
                err!("Failed to render emoji")
            })?;
        } else {
            let font_to_use = font_map(c.clone()).await;

            let scaled_font = font_to_use.as_scaled(font_scale);

            draw_text_mut(image, color, cursor_x as i32, y, font_scale, &font_to_use, c.as_str());

            // 根据glyph计算字符的宽度，用于更新cursor_x
            for x in c.chars() {
                let glyph = scaled_font.scaled_glyph(x);
                let h_metrics = scaled_font.h_advance(glyph.id);
                cursor_x += h_metrics;
            }
        }
    }

    Ok(())

}

pub fn draw_filled_rect_with_circle_corner(image: &mut RgbaImage, rect: Rect, color: Rgba<u8>, radius: u32) {
    let width = rect.width();
    let height = rect.height();
    assert!(radius * 2 <= height, "Radius is too large for the rectangle");
    if radius * 2 > width {
        // 太小就不要圆角了嘛
        draw_filled_rect_mut(image, rect, color);
        return;
    }
    let (left, top) = (rect.left(), rect.top());
    let radius_i32 = radius as i32;

    // 绘制交叉矩形
    draw_filled_rect_mut(image, Rect::at(left + radius_i32, top).of_size(width - radius * 2, height),  color);
    draw_filled_rect_mut(image, Rect::at(left, top + radius_i32 ).of_size(width, height - radius * 2),  color);

    // 绘制四个圆角
    draw_filled_circle_mut(image, (left + radius_i32, top + radius as i32), radius_i32, color);
    draw_filled_circle_mut(image, (left + width as i32 - radius_i32 - 1, top + radius as i32), radius_i32, color);
    draw_filled_circle_mut(image, (left + radius_i32, top + height as i32 - radius_i32 - 1), radius_i32, color);
    draw_filled_circle_mut(image, (left + width as i32 - radius_i32 - 1, top + height as i32 - radius_i32 - 1), radius_i32, color);
}


pub fn get_text_size(font: &FontRef, text: &str, scale: PxScale) -> (f32, f32) {
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