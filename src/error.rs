#[derive(Debug)]
pub struct Error(String);

pub fn err<S: Into<String>>(s: S) -> Error {
    Error(s.into())
}

impl std::convert::From<std::option::NoneError> for Error {
    fn from(_: std::option::NoneError) -> Error {
        Error("Something went wrong!".to_string())
    }
}
impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error(format!("{}", err))
    }
}
impl <'a> std::convert::From<&'a str> for Error {
    fn from(err: &'a str) -> Error {
        Error(format!("{}", err))
    }
}