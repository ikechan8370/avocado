use std::fmt::Display;
use image::ImageError;
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
    Network
}

impl Error{
    pub fn new(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Internal
        }
    }
    pub fn kritor(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Kritor
        }
    }

    pub fn client(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Client
        }
    }

    pub fn network(msg: String) -> Self {
        Error {
            msg,
            kind: Kind::Network
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
            kind: Kind::Internal
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        let msg = format!("{}", e);
        Error {
            msg,
            kind: Kind::Network
        }
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        let msg = format!("{}", e);
        Error {
            msg,
            kind: Kind::Internal
        }
    }
}

impl From<ZipError> for Error {
    fn from(value: ZipError) -> Self {
        let msg = format!("{}", value);
        Error {
            msg,
            kind: Kind::Internal
        }
    }
}

#[macro_export]
macro_rules! err {
    ($x:expr) => {
        Err(crate::model::error::Error::new($x.to_string()))
    };
}

#[macro_export]
macro_rules! kritor_err {
    ($x:expr) => {
        Err(crate::model::error::Error::kritor($x.to_string()))
    };
}

#[macro_export]
macro_rules! client_err {
    ($x:expr) => {
        Err(crate::model::error::Error::client($x.to_string()))
    };
}

#[macro_export]
macro_rules! network_err {
    ($x:expr) => {
        Err(crate::model::error::Error::network($x.to_string()))
    };
}