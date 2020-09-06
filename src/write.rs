use std::io::Write;

pub struct JsonSeqWriter<W: Write> {
    inner: W,
}

impl<W: Write> JsonSeqWriter<W> {

    /// Construct a new JsonSeqWriter from a `Write`
    pub fn new(inner: W) -> Self {
        JsonSeqWriter { inner }
    }

    /// Write raw bytes to this writer
    fn write_item_raw(&mut self, data: &[u8]) -> std::io::Result<()> {
        assert!(data.iter().all(|byte| *byte != 0x1E));
        self.inner.write_all(&[0x1E])?;
        self.inner.write_all(data)?;
        self.inner.write_all(&[0x0A])?;

        Ok(())
    }

    /// Return a reference to the inner `Write`
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Return a mutable reference to the inner `Write`
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Consume this and return the inner `Write`
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Write a single serde_json object to this
    pub fn write_item(&mut self, value: &serde_json::Value) -> std::io::Result<()> {
        self.write_item_raw(&serde_json::to_vec(value)?)
    }
}

impl<W: Write> From<W> for JsonSeqWriter<W> {
    fn from(wtr: W) -> Self {
        Self::new(wtr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw1() {
        let mut buf = Vec::new();
        let mut wtr = JsonSeqWriter::new(buf);

        wtr.write_item_raw("hello".as_bytes()).unwrap();

        assert_eq!(
            wtr.get_ref(),
            &[0x1E, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0x0A]
        );

    }

    #[test]
    fn json1() {
        use serde_json::json;

        let buf = Vec::new();
        let mut wtr = JsonSeqWriter::new(buf);

        wtr.write_item(&json!({"foo": "bar"})).unwrap();
        wtr.write_item(&json!([1, 2, "c"])).unwrap();

        let buf = wtr.into_inner();

        let (step, buf) = buf.split_at(1);
        assert_eq!(step, &[0x1E]);
        let (step, buf) = buf.split_at(13);
        assert_eq!(step, "{\"foo\":\"bar\"}".as_bytes());
        let (step, buf) = buf.split_at(1);
        assert_eq!(step, &[0x0A]);

        let (step, buf) = buf.split_at(1);
        assert_eq!(step, &[0x1E]);
        let (step, buf) = buf.split_at(9);
        assert_eq!(step, "[1,2,\"c\"]".as_bytes());
        let (step, buf) = buf.split_at(1);
        assert_eq!(step, &[0x0A]);

        assert_eq!(buf.len(), 0);

    }
}
