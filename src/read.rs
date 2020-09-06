use std::io::{BufRead, BufReader, Cursor, Read};

use crate::Error;

/// Reads data as JSON sequences
pub struct JsonSeqReader<R: Read> {
    inner: BufReader<R>,
}

impl<R: Read> JsonSeqReader<R> {
    /// Construct from a `Read`
    pub fn new(inner: R) -> Self {
        JsonSeqReader {
            inner: BufReader::new(inner),
        }
    }

    /// Helper method to create from a &str
    pub fn new_from_str<'a>(s: &'a str) -> JsonSeqReader<Cursor<&'a [u8]>> {
        JsonSeqReader::new(Cursor::new(s.as_bytes()))
    }

    /// Helper method to create from a &[u8]
    pub fn new_from_slice(s: &[u8]) -> JsonSeqReader<Cursor<&[u8]>> {
        JsonSeqReader::new(Cursor::new(s))
    }

    /// Helper method to create from a String
    pub fn new_from_string(s: String) -> JsonSeqReader<Cursor<Vec<u8>>> {
        JsonSeqReader::new(Cursor::new(s.into_bytes()))
    }

    /// Reads the next item from 
    fn next_item_raw(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        let mut buf = Vec::new();
        loop {
            buf.clear();
            let num_read = self.inner.read_until(0x1E, &mut buf)?;
            //dbg!(&buf);
            if num_read == 0 {
                // EOF
                return Ok(None);
            } else if num_read == 1 && buf == [0x1E] {
                continue;
            } else {
                if buf.last() == Some(&0x1E) {
                    // If we get to EOF, then the last char won't be the RS
                    buf.pop();
                }
                return Ok(Some(buf));
            }
        }
    }

    fn next_item_str(&mut self) -> std::io::Result<Option<String>> {
        self.next_item_raw()
            .map(|bytes_opt| bytes_opt.map(|bytes| String::from_utf8(bytes).unwrap()))
    }

    fn iter_str(&mut self) -> JsonSeqReaderStringIter<R> {
        JsonSeqReaderStringIter(self)
    }

    fn iter_bytes(&mut self) -> JsonSeqReaderBytesIter<R> {
        JsonSeqReaderBytesIter(self)
    }

    /// Reads & returns the next JSON object.
    pub fn next_from_json(&mut self) -> Result<Option<serde_json::Value>, Error> {
        let res = self.next_item_raw()?;
        match res {
            None => Ok(None),
            Some(bytes) => {
                let serde_v = serde_json::from_slice(&bytes)?;
                Ok(Some(serde_v))
            }
        }
    }

    /// Return a reference to the inner `Read`
    pub fn get_ref(&self) -> &R {
        &self.inner.get_ref()
    }

    /// Return a mutable reference to the inner `Read`
    pub fn get_mut(&mut self) -> &mut R {
        self.inner.get_mut()
    }

    /// Consume, and return the inner `Read`
    pub fn into_inner(self) -> R {
        self.inner.into_inner()
    }
}

impl<R: Read> From<R> for JsonSeqReader<R> {
    fn from(rdr: R) -> Self {
        Self::new(rdr)
    }
}

impl<R: Read> Iterator for JsonSeqReader<R> {
    type Item = Result<serde_json::Value, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_from_json().transpose()
    }
}

struct JsonSeqReaderBytesIter<'a, R: Read>(&'a mut JsonSeqReader<R>);

impl<'a, R: Read> Iterator for JsonSeqReaderBytesIter<'a, R>
{
    type Item = std::io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_item_raw().transpose()
    }
}

struct JsonSeqReaderStringIter<'a, R: Read>(&'a mut JsonSeqReader<R>);

impl<'a, R: Read> Iterator for JsonSeqReaderStringIter<'a, R> {
    type Item = std::io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_item_str().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    macro_rules! assert_input_output {
        ( $name:ident, $input:expr, $expected_output:expr) => {
            #[test]
            fn $name() {
                let mut rdr = JsonSeqReader::new(Cursor::new($input));
                let output = rdr.iter_str().collect::<Result<Vec<String>, _>>().unwrap();
                assert_eq!(output, $expected_output);
            }
        };
    }

    assert_input_output!(empty1, "\x1E", &[] as &[&str]);
    assert_input_output!(empty2, "\x1E\x1E", &[] as &[&str]);
    assert_input_output!(simple1, "\x1E{}", &["{}"]);
    assert_input_output!(simple2, "\x1E\x1E{}", &["{}"]);
    assert_input_output!(simple3, "\x1Efoo\x1Ebar", &["foo", "bar"]);
    assert_input_output!(simple4, "\x1Efoo\x1Ebar\x1E", &["foo", "bar"]);

    // Spec says consequtive \x1E's must be ignored
    assert_input_output!(simple5, "\x1Ea\x1E\x1E\x1Eb\x1E", &["a", "b"]);

    assert_input_output!(nl1, "\x1Efoo\n\x1Ebar\n", &["foo\n", "bar\n"]);

    macro_rules! assert_input_output_json {
        ( $name:ident, $input:expr, $expected_output:expr) => {
            #[test]
            fn $name() {
                let rdr = JsonSeqReader::new(Cursor::new($input));
                let output = rdr
                    .into_iter()
                    .collect::<Result<Vec<serde_json::Value>, _>>()
                    .unwrap();
                assert_eq!(output, $expected_output);
            }
        };
    }

    assert_input_output_json!(json1, "\x1E[]\x1E[]\x1E", &[json!([]), json!([]),]);

    assert_input_output_json!(
        json2,
        "\x1E{}\x1E{\"foo\": 3}\x1E",
        &[json!({}), json!({"foo": 3}),]
    );

    assert_input_output_json!(json_with_nl1, "\x1E{}\x0A", &[serde_json::json!({}),]);
    assert_input_output_json!(
        json_with_nl2,
        "\x1E{}\x0A\x1E{\"baz\":[]}\x0A",
        &[json!({}), json!({"baz": []}),]
    );
}
