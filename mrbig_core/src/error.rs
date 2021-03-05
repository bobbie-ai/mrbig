#[derive(Debug)]
pub enum Inner {
    // io error
    Io(std::io::Error),
    /// Deserialization error
    De(toml::de::Error),
    /// getopts error
    Opts(getopts::Fail),
    /// Address parse error
    Addr(std::net::AddrParseError),
    /// Other kind of error
    Other(String),
}

/// Custom error type to encapsulate other errors.
#[derive(Debug)]
pub struct Error {
    /// Inner error type and source.
    pub inner: Inner,
}

impl Error {
    /// Create a new error from a message. The inner error is of type `Inner::Other`.
    pub fn new(message: &str) -> Self {
        Error {
            inner: Inner::Other(message.into()),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.inner {
            Inner::Io(e) => write!(f, "io error: {}", e.to_string()),
            Inner::De(e) => write!(f, "deserialization error: {}", e.to_string()),
            Inner::Opts(e) => write!(f, "options error: {}", e.to_string()),
            Inner::Addr(e) => write!(f, "address parse error: {}", e.to_string()),
            e => write!(f, "{:?}", e),
        }
    }
}

impl From<Error> for String {
    fn from(e: Error) -> String {
        e.to_string()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.inner {
            Inner::Io(ref e) => Some(e),
            Inner::De(ref e) => Some(e),
            Inner::Opts(ref e) => Some(e),
            Inner::Addr(ref e) => Some(e),
            Inner::Other(_) => None,
        }
    }
}

macro_rules! error_from {
    ($errty:ty, $errx:path) => {
        impl From<$errty> for Error {
            fn from(error: $errty) -> Self {
                Error {
                    inner: $errx(error),
                }
            }
        }
    };
}

error_from! { std::io::Error, Inner::Io }
error_from! { toml::de::Error, Inner::De }
error_from! { getopts::Fail, Inner::Opts }
error_from! { std::net::AddrParseError, Inner::Addr }
error_from! { String, Inner::Other }
