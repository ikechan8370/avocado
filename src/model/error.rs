use image::ImageError;
use std::fmt::Display;
use zip::result::ZipError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
    kind: Kind,
}

#[derive(Debug, Clone)]
pub enum Kind {
    Internal,
    Kritor,
    Client,
    Network,
}

impl Error {
    pub fn new(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Internal,
        }
    }
    pub fn kritor(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Kritor,
        }
    }

    pub fn client(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Client,
        }
    }

    pub fn network(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Network,
        }
    }
    pub fn error(&self) -> String {
        self.msg.clone()
    }

    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg.clone())
    }
}

impl From<tonic::Status> for Error {
    fn from(e: tonic::Status) -> Self {
        let msg = format!("{}", e);
        Error {
            msg,
            kind: Kind::Internal,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        let msg = format!("{}", e);
        Error {
            msg,
            kind: Kind::Network,
        }
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        let msg = format!("{}", e);
        Error {
            msg,
            kind: Kind::Internal,
        }
    }
}

impl From<ZipError> for Error {
    fn from(value: ZipError) -> Self {
        let msg = format!("{}", value);
        Error {
            msg,
            kind: Kind::Internal,
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        let msg = format!("{}", e);
        Error {
            msg,
            kind: Kind::Internal,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        let msg = format!("{}", e);
        Error {
            msg,
            kind: Kind::Internal,
        }
    }
}


impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        let msg = format!("{}", value);
        Error {
            msg,
            kind: Kind::Internal,
        }
    }
}

#[macro_export]
macro_rules! err {
    ($msg:expr) => {
        Err(crate::model::error::Error::new(String::from($msg)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        Err(crate::model::error::Error::new(format!($fmt, $($arg)*)))
    };
}

#[macro_export]
macro_rules! kritor_err {
    ($msg:expr) => {
        Err(crate::model::error::Error::kritor(String::from($msg)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        Err(crate::model::error::Error::kritor(format!($fmt, $($arg)*)))
    };
}

#[macro_export]
macro_rules! client_err {
    ($msg:expr) => {
        Err(crate::model::error::Error::client(String::from($msg)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        Err(crate::model::error::Error::client(format!($fmt, $($arg)*)))
    };
}

#[macro_export]
macro_rules! network_err {
     ($msg:expr) => {
        Err(crate::model::error::Error::network(String::from($msg)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        Err(crate::model::error::Error::network(format!($fmt, $($arg)*)))
    };
}
