extern crate serde;
pub extern crate serde_json;

mod read;
mod write;

pub use read::JsonSeqReader;
pub use write::JsonSeqWriter;

/// An error when reading or writing
#[derive(Debug)]
pub enum Error {
    /// An underlying IO error
    IOError(std::io::Error),

    /// Error when decode to/from JSON.
    JsonError(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::JsonError(e)
    }
}
