use crate::model::error::Result;

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

