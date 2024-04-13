use crate::model::error::Result;
use chrono::{Local, Datelike, Timelike, Weekday};

pub fn format_duration(duration: u64) -> Result<String> {

    let mut duration_str = String::new();
    let days = duration / 86400;
    if days > 0 {
        duration_str.push_str(&format!("{}天", days));
    }
    let hours = (duration % 86400) / 3600;
    if hours > 0 {
        duration_str.push_str(&format!("{}小时", hours));
    }
    let minutes = (duration % 3600) / 60;
    if minutes > 0 {
        duration_str.push_str(&format!("{}分钟", minutes));
    }
    let seconds = duration % 60;
    if seconds > 0 {
        duration_str.push_str(&format!("{}秒", seconds));
    }

    if duration_str.is_empty() {
        return Ok("0秒".to_string());
    }

    Ok(duration_str)
}

pub fn now_format() -> String {
    let local_time = Local::now();

    let year = local_time.year();
    let month = local_time.month();
    let day = local_time.day();
    let hour = local_time.hour();
    let minute = local_time.minute();
    let weekday = local_time.weekday();

    // 星期几的中文表达
    let weekday_str = match weekday {
        Weekday::Mon => "星期一",
        Weekday::Tue => "星期二",
        Weekday::Wed => "星期三",
        Weekday::Thu => "星期四",
        Weekday::Fri => "星期五",
        Weekday::Sat => "星期六",
        Weekday::Sun => "星期日",
    };

    format!(
        "{}年{}月{}日 {}:{} {}",
        year, month, day, hour, minute, weekday_str
    )
}