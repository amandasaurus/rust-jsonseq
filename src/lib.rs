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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use serde_json::json;

    #[test]
    fn roundtrip1() {
        let buf = Vec::new();
        let mut wtr = JsonSeqWriter::new(buf);

        wtr.write_item(&json!({"foo": "bar"})).unwrap();
        wtr.write_item(&json!([1, 2, "c"])).unwrap();

        let buf = wtr.into_inner();
        let mut rdr = JsonSeqReader::new(Cursor::new(buf));

        assert_eq!(rdr.read_item().unwrap().unwrap(), json!({"foo": "bar"}));
        assert_eq!(rdr.read_item().unwrap().unwrap(), json!([1, 2, "c"]));
        assert_eq!(rdr.read_item().unwrap(), None);
        assert_eq!(rdr.read_item().unwrap(), None);
    }

}
