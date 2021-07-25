#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    AlsaError { message: String, sys_error: String },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IOError(error)
    }
}
