use std::cmp::max;
use std::io::Cursor;
use std::time::SystemTime;
use std::vec;

use ab_glyph::{FontRef, PxScale};
use async_trait::async_trait;
use image::{Rgba, RgbaImage};
use image::imageops::FilterType;
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;
use sysinfo::{CpuRefreshKind, Disks, Networks, Pid, RefreshKind, System};

use avocado_common::Event;
use avocado_macro::service;

use crate::image;
use crate::service::service::Elements;
use crate::service::service::{KritorContext, Service};
use crate::utils::common::bytes_to_readable_string;
use crate::utils::common::memory::get_current_memory_usage;
use crate::utils::image::{DEFAULT_NORMAL_FONT, draw_filled_rect_with_circle_corner, get_text_size, overlay_image, overlay_image_from_url, OverlayImageOption, render_text_with_different_fonts};
use crate::utils::time::{format_duration, now_format};

#[derive(Debug, Clone, Default)]
#[service(
    name = "status",
    pattern = "^([!！])(status|Status|STATUS|状态)$",
    events(Event::Message)
)]
struct StatusService;


#[async_trait]
impl Service for StatusService {
    // fn matches(&self, context: KritorContext) -> bool {
    //     let re = Regex::new(r"^([!！])(status|Status|STATUS|状态)$").unwrap();
    //     if let Some(message) = context.message {
    //         if let Some(elements) = message.elements.get_text_elements() {
    //             return elements.iter().any(|ele| re.is_match(ele.text.as_str()));
    //         }
    //     }
    //     false
    // }

    async fn process(&self, context: KritorContext) {
        // let text = {
        //     let bot = context.bot.read().await;
        //     let nickname = context.message.as_ref().and_then(|m| m.sender.as_ref().and_then(|s| s.nick.as_ref())).cloned().unwrap_or_default();
        //     let uin = context.message.as_ref().and_then(|m| m.sender.as_ref().and_then(|s| s.uin.as_ref())).cloned().unwrap_or_default();
        //     let uid = context.message.as_ref().and_then(|m| m.sender.as_ref().map(|s| s.uid.as_str())).unwrap_or_default();
        //
        //     let more = match context.message.as_ref().and_then(|m| m.contact.as_ref().map(|c| c.scene())) {
        //         Some(Scene::Group) => format!("群号: {}", context.message.as_ref().unwrap().contact.as_ref().unwrap().peer),
        //         Some(Scene::Friend) => format!("私聊对象: {}", context.message.as_ref().unwrap().contact.as_ref().unwrap().peer),
        //         _ => "".to_string(),
        //     };
        //     let mut text = format!("发送者信息\nnickname: {}\nuin: {}\nuid: {}\n{}", nickname, uin, uid, more);
        //
        //     let start_time = bot.get_uptime();
        //     let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() - start_time;
        //     let duration_str = format_duration(duration).unwrap_or("刚刚启动".to_string());
        //
        //     let version = bot.get_kritor_version().unwrap_or("未知".to_string());
        //     let groups =  bot.get_groups();
        //     let groups = groups.read().await;
        //     let group_num = groups.as_ref().unwrap().len();
        //     let friends = bot.get_friends();
        //     let friends = friends.read().await;
        //     let friend_num = friends.as_ref().unwrap().len();
        //
        //     text = text + format!("\n\n运行状态\n运行时间：{}\n协议版本：{}\n已发送：{}\n已接收：{}\n群总数：{}\n好友总数：{}\n\nPowered by avocado-rs and kritor with ❤",
        //                           duration_str, version, bot.get_sent(), bot.get_receive(), group_num, friend_num).as_str();
        //     text
        // };
        // context.reply_with_quote(vec![text!(text)]).await.unwrap();

        context.reply(vec![image!(draw(&context).await)]).await.unwrap();
    }
}

async fn draw(context: &KritorContext) -> Vec<u8> {
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
    );

    let bot = context.bot.read().await;
    let start_time = bot.get_uptime();
    let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() - start_time;
    let duration_str = format_duration(duration).unwrap_or("刚刚启动".to_string());

    let version = bot.get_kritor_version().unwrap_or("未知".to_string());
    let groups =  bot.get_groups_arc();
    let groups = groups.read().await;
    let group_num = groups.as_ref().unwrap().len();
    let friends = bot.get_friends_arc();
    let friends = friends.read().await;
    let friend_num = friends.as_ref().unwrap().len();
    let sent = bot.get_sent();
    let receive = bot.get_receive();
    let nickname = bot.get_nickname().await.unwrap_or("unknown".to_string());
    let uin = bot.get_uin().unwrap_or_default();

    let client_version = bot.get_client_version().await.unwrap_or_default();
    drop(bot);

    // 本进程占用
    let self_id = std::process::id();
    let current_memory_usage = get_current_memory_usage(Some(self_id)).unwrap_or(0);

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

    let x = padding_left_right;
    let y = 30;
    let theme_color = Rgba([84u8, 128u8, 2u8, 255u8]);
    overlay_image((padding_left_right, y), &mut image, &overlay_img, OverlayImageOption::new()).unwrap();

    // draw_text_mut(&mut image, theme_color, ((width - w - height as u32) / 2 + height as u32) as i32, 30, scale, &font, text);
    draw_bold_weight(&mut image, 30, x + height as u32, theme_color, scale, &font, text);

    let date_scale = PxScale {
        x: height * 0.5,
        y: height * 0.5,
    };
    let date = now_format();
    let (date_w, date_h) = text_size(date_scale, &font, date.as_str());

    draw_text_mut(&mut image, grey, (width - padding_left_right - date_w) as i32, y as i32 + (0.45 * height) as i32, date_scale, &font, &date);

    let padding = 30;

    let _ = draw_line_segment_mut(&mut image, (padding_left_right as f32, (h + y + padding) as f32), ((width as i32 - 50) as f32, (h + y + padding) as f32), theme_color);

    // icon
    let x = padding_left_right;
    let y = 30 + h + padding + 30;
    let crop_height = width / 3;
    overlay_image_from_url((x, y), &mut image, format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=0", uin),
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

    // draw_text_mut(&mut image, black, (icon_right_x + 20) as i32, y as i32, scale, &font, text);
    render_text_with_different_fonts(&mut image, Rgba([88, 214, 141, 255]), text_left_x, current_y, scale, nickname, None).await.unwrap();
    current_y += font_size as i32;



    let font_size = 40;
    let scale = PxScale {
        x: font_size as f32,
        y: font_size as f32,
    };
    let line_height = 10;
    current_y += line_height * 2;

    let text = format!("💻 本次在线：{}", duration_str);
    render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text, None).await.unwrap();
    current_y += font_size + line_height;

    let text = format!("👨‍👩‍👧‍👧 群聊：{} 🧑好友：{}", group_num, friend_num);
    render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text, None).await.unwrap();
    current_y += font_size + line_height;

    let text = format!("📧 发送：{} 📬接收：{}", sent, receive);
    render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text, None).await.unwrap();
    current_y += font_size + line_height;

    let text = format!("❤️‍🔥 Powered by {}", version);
    render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text, None).await.unwrap();
    current_y += font_size + line_height;

    let text = format!("📱 {}", client_version);
    render_text_with_different_fonts(&mut image, black, text_left_x, current_y, scale, text, None).await.unwrap();
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
    // let blue = Rgba([0u8, 0u8, 255u8, 255u8]);
    let content_distant = 4;
    let content_height = (bar_height - content_distant * 2) as u32;


    let content_font_size = 36;
    let content_scale = PxScale {
        x: content_font_size as f32,
        y: content_font_size as f32,
    };



    // CPU
    sys.refresh_all();
    let cpu_num = sys.cpus().len();
    let total = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).reduce(|a, b| a + b).unwrap_or(0.);
    let cpu_usage = total / sys.cpus().len() as f32;
    let cpu = sys.cpus().get(0).unwrap();
    let cpu_name = cpu.brand();
    let text = "CPU: ";
    draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
    draw_bar(&mut image, current_y, format!("{:.1}%", cpu_usage).as_str(), cpu_usage as f64 / 100f64, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_scale, white);
    current_y += font_size + 20;

    // Memory
    let total_memory = sys.total_memory();
    let available_memory = sys.available_memory();
    let rate = (total_memory - available_memory) as f64 / total_memory as f64;
    let text = "内存: ";
    draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
    draw_bar(&mut image, current_y, format!("{:.1}%", rate * 100f64).as_str(), rate, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_scale, white);
    current_y += font_size + 20;

    // swap
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    let swap_rate = used_swap as f64 / total_swap as f64;
    let text = "交换: ";
    draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
    draw_bar(&mut image, current_y, format!("{:.1}%", swap_rate * 100f64).as_str(), swap_rate, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_scale, white);
    current_y += font_size + 20;

    // Disk
    let disks = Disks::new_with_refreshed_list();
    let mut total = 0;
    let mut available = 0;
    for disk in &disks {
        total += disk.total_space();
        available += disk.available_space();
    }
    let disk_rate = (total - available) as f64 / total as f64;
    let text = "硬盘: ";
    draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);
    draw_bar(&mut image, current_y, format!("{:.1}%", disk_rate * 100f64).as_str(), disk_rate, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, content_distant, content_height, content_scale, white);
    current_y += font_size + 20;

    let networks = Networks::new_with_refreshed_list();
    let (_interface_name, data) = networks.iter().max_by(|a, b| a.1.total_received().partial_cmp(&b.1.total_received()).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
    let up = data.total_transmitted();
    let down = data.total_received();
    let text = "网络: ";
    let net_width = (width - padding_left_right * 2) * 3 / 5;
    let net_content_width = net_width - estimated_title_width;
    draw_bold_weight(&mut image, current_y, padding_left_right, black, scale, &font, text);

    // draw_bar(&mut image, current_y, "98%", 0.98, &font, font_size, bar_height, bar_bg_color, bar_radius, chart_start_x, bar_width, blue, content_distant, content_height, content_font_size as f32, content_scale, white);
    let up_text = format!("⬆ {}", bytes_to_readable_string(up));
    let down_text = format!("⬇ {}", bytes_to_readable_string(down));
    let network_font_size = 42;
    let network_scale = PxScale {
        x: network_font_size as f32,
        y: network_font_size as f32,
    };
    let net_start_x = padding_left_right + estimated_title_width;
    let network_text_top = current_y  + (estimated_title_height - network_font_size) / 2;
    render_text_with_different_fonts(&mut image, black, net_start_x as i32, network_text_top, network_scale, up_text.to_string(), None).await.unwrap();
    render_text_with_different_fonts(&mut image, black, net_start_x as i32 + (net_content_width / 2) as i32, network_text_top, network_scale, down_text.to_string(), None).await.unwrap();

    let text = "在线: ";
    let uptime_left = padding_left_right + net_width;
    let uptime_content_left = uptime_left + estimated_title_width;
    draw_bold_weight(&mut image, current_y, uptime_left, black, scale, &font, text);
    let uptime_font_size = 48;
    let uptime_scale = PxScale {
        x: uptime_font_size as f32,
        y: uptime_font_size as f32,
    };
    let uptime = System::uptime();
    let uptime_text = format_duration(uptime).unwrap_or("刚刚启动".to_string());
    // let uptime_content_width = width - padding_left_right * 2 - uptime_content_left;
    let (content_text_width, content_text_height) = get_text_size(&font, uptime_text.as_str(), uptime_scale);
    // 贴着好了
    let uptime_text_left = uptime_content_left + 5;
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

    let text = format!("🖥  操作系统 —— {}", System::long_os_version().unwrap_or_else(|| "<unknown>".to_owned()));
    render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
    current_y += brand_font_size + 20;

    let text = format!("🏅  主机名称 —— {}", System::name().unwrap_or_else(|| "<unknown>".to_owned()));
    render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
    current_y += brand_font_size + 20;

    let text = format!("🧿  内核 —— {}", System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()));
    render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
    current_y += brand_font_size + 20;

    let text = format!("💾  CPU —— {} * {}", cpu_name.trim(), cpu_num);
    render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
    current_y += brand_font_size + 20;

    let text = format!("🧠  内存 —— {} / {}", bytes_to_readable_string(total_memory - available_memory), bytes_to_readable_string(total_memory));
    render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
    current_y += brand_font_size + 20;

    let text = format!("🪧  进程总数 —— {}", sys.processes().len());
    render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
    current_y += brand_font_size + 20;

    if let Some(self_process) = sys.process(Pid::from_u32(self_id)) {
        let text = format!("🥑  本进程占用 —— CPU {:.1}% 内存 {}", self_process.cpu_usage(), bytes_to_readable_string(current_memory_usage as u64));
        render_text_with_different_fonts(&mut image, color, padding_left_right as i32, current_y, scale, text.to_string(), None).await.unwrap();
        current_y += brand_font_size + 20;
    };

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
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageFormat::Png).unwrap();
    let image_bytes: Vec<u8> = buffer.into_inner();
    // let b64 = general_purpose::STANDARD.encode(image_bytes.as_slice()).as_str().to_string();
    // info!("base64: {}", b64);
    image_bytes
}


fn draw_bar(image: &mut RgbaImage, current_y: i32, content_text: &str, rate: f64, font: &FontRef, font_size: i32, bar_height: i32, bar_bg_color: Rgba<u8>, bar_radius: u32, chart_start_x: u32, bar_width: u32, content_distant: i32, content_height: u32, content_scale: PxScale, text_color: Rgba<u8>) {
    draw_filled_rect_with_circle_corner(image, Rect::at(chart_start_x as i32, current_y + (font_size - bar_height) / 2).of_size(bar_width, bar_height as u32), bar_bg_color, bar_radius);
    let content_length = ((bar_width as i32 - 2 * content_distant) as f64 * rate) as u32;
    let content_left = chart_start_x as i32 + content_distant;
    let content_top = current_y + (font_size - bar_height) / 2 + content_distant;
    let content_color = match rate {
        r if r < 0.5 => Rgba([88u8, 214u8, 141u8, 255u8]),
        r if r < 0.8 => Rgba([70u8, 173u8, 255u8, 255u8]),
        _ => Rgba([199, 0, 57, 255u8]),
    };
    if rate > 0f64 {
        draw_filled_rect_with_circle_corner(image, Rect::at(content_left, content_top).of_size(content_length, content_height), content_color, bar_radius - (content_distant / 2) as u32);
        let (content_text_width, content_text_height) = get_text_size(font, content_text, content_scale);
        let content_text_top = content_top + ((content_height as f32 - content_text_height) / 2.) as i32;
        let content_tet_left = content_left + ((content_length as f32 - content_text_width) / 2.) as i32;
        draw_text_mut(image, text_color, content_tet_left, content_text_top, content_scale, &font, content_text);
    } else {
        let (content_text_width, content_text_height) = get_text_size(font, content_text, content_scale);
        let content_text_top = content_top + ((content_height as f32 - content_text_height) / 2.) as i32;
        let content_tet_left = content_left + ((bar_width as f32 - content_text_width) / 2.) as i32;
        draw_text_mut(image, text_color, content_tet_left, content_text_top, content_scale, &font, content_text);
    }
}

fn draw_bold_weight(image: &mut RgbaImage, current_y: i32, padding_left_right: u32, black: Rgba<u8>, scale: PxScale, font: &FontRef, text: &str) {
    for offset_x in &[0.0, 1.0, -1.0] {
        for offset_y in &[0.0, 1.0, -1.0]  {
            draw_text_mut(image, black, (padding_left_right as f64 + offset_x.clone()) as i32, (current_y as f64 + offset_y.clone()) as i32, scale, &font, text);
        }
    }
}