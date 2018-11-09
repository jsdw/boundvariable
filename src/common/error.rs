#[derive(Debug)]
pub struct Error(String);

pub fn err<S: Into<String>>(s: S) -> Error {
    Error(s.into())
}

// quick macro to stringify an error into our Error type:
macro_rules! err_from {
    ($ty:ty) => {
        impl std::convert::From<$ty> for Error {
            fn from(err: $ty) -> Error {
                Error(format!("{}", err))
            }
        }
    }
}

err_from!(futures::sync::mpsc::SendError<u8>);
err_from!(std::net::AddrParseError);
err_from!(std::io::Error);
err_from!(&str);

impl std::convert::From<std::option::NoneError> for Error {
    fn from(_: std::option::NoneError) -> Error {
        Error("Something went wrong!".to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}