use std::fmt;
use std::io::{self, BufRead, BufReader, Read};

use crate::line_item::{LineItem, ParseError};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Parse { line: u64, source: ParseError },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "io error: {}", e),
            ReadError::Parse { line, source } => write!(f, "line {}: {}", line, source),
        }
    }
}

impl std::error::Error for ReadError {}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

/// Reads invoice line items one at a time from any `Read` source.
///
/// The whole input is never buffered at once. Each call to `next()` pulls
/// one line off the underlying reader into a reusable buffer, parses it,
/// and hands back the result before touching the next line - a
/// multi-gigabyte export costs about the same memory as a ten-line one.
pub struct LineItemReader<R> {
    inner: BufReader<R>,
    buf: String,
    line_no: u64,
}

impl<R: Read> LineItemReader<R> {
    pub fn new(source: R) -> Self {
        LineItemReader {
            inner: BufReader::new(source),
            buf: String::new(),
            line_no: 0,
        }
    }
}

impl<R: Read> Iterator for LineItemReader<R> {
    type Item = Result<LineItem, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            let bytes_read = match self.inner.read_line(&mut self.buf) {
                Ok(n) => n,
                Err(e) => return Some(Err(ReadError::Io(e))),
            };
            if bytes_read == 0 {
                return None;
            }
            self.line_no += 1;

            if self.buf.trim().is_empty() {
                continue;
            }

            return Some(LineItem::parse(&self.buf).map_err(|source| ReadError::Parse {
                line: self.line_no,
                source,
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterates_multiple_lines_and_skips_blanks() {
        let input = "widget, 3, 1250, 3750\n\ngadget, 1, 999, 999\n";
        let reader = LineItemReader::new(input.as_bytes());
        let items: Vec<LineItem> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].description, "widget");
        assert_eq!(items[1].description, "gadget");
    }

    #[test]
    fn reports_the_line_number_of_a_bad_row() {
        let input = "widget, 3, 1250, 3750\nbroken row\n";
        let reader = LineItemReader::new(input.as_bytes());
        let results: Vec<_> = reader.collect();
        match &results[1] {
            Err(ReadError::Parse { line, .. }) => assert_eq!(*line, 2),
            other => panic!("expected a parse error, got {:?}", other),
        }
    }
}
