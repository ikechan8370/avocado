#[derive(Debug)]
pub enum Event {
    Notice,
    Message,
    Request,
}

impl From<&str> for Event {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "message" => Self::Message,
            "notice" => Self::Notice,
            "request" => Self::Request,
            _ => Self::Message
        }
    }
}