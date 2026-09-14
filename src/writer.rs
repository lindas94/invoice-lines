use std::io::{self, Write};

use crate::line_item::LineItem;

/// Writes invoice line items one at a time to any `Write` sink, in the
/// same format [`LineItemReader`](crate::LineItemReader) reads.
///
/// Each call to `write_item` formats and writes a single line - nothing
/// is held in memory beyond the one item being written, so producing a
/// large export costs about the same memory as producing a small one.
pub struct LineItemWriter<W> {
    inner: W,
}

impl<W: Write> LineItemWriter<W> {
    pub fn new(sink: W) -> Self {
        LineItemWriter { inner: sink }
    }

    /// Writes one line item followed by a newline.
    pub fn write_item(&mut self, item: &LineItem) -> io::Result<()> {
        self.inner.write_all(item.to_line().as_bytes())?;
        self.inner.write_all(b"\n")
    }

    /// Flushes the underlying sink.
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_multiple_items_one_line_each() {
        let items = vec![
            LineItem::parse("widget, 3, 1250, 3750").unwrap(),
            LineItem::parse("gadget, 1, 999, 999").unwrap(),
        ];
        let mut buf: Vec<u8> = Vec::new();
        let mut writer = LineItemWriter::new(&mut buf);
        for item in &items {
            writer.write_item(item).unwrap();
        }
        writer.flush().unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "widget,3,1250,3750\ngadget,1,999,999\n");
    }

    #[test]
    fn round_trips_through_writer_and_reader() {
        use crate::reader::LineItemReader;

        let items = vec![
            LineItem::parse("\"widgets, deluxe\", 3.5, 1250, 4375").unwrap(),
            LineItem::parse("\"6\"\" pipe\", 2, 500, 1000").unwrap(),
        ];
        let mut buf: Vec<u8> = Vec::new();
        let mut writer = LineItemWriter::new(&mut buf);
        for item in &items {
            writer.write_item(item).unwrap();
        }

        let read_back: Vec<LineItem> = LineItemReader::new(buf.as_slice())
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(read_back, items);
    }
}
