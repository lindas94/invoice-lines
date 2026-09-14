//! Parses invoice line item exports without loading the whole file into memory.
//!
//! The core type is [`LineItemReader`], which wraps anything implementing
//! `std::io::Read` and yields one [`LineItem`] at a time.

mod line_item;
mod reader;
mod writer;

pub use line_item::{LineItem, ParseError};
pub use reader::{LineItemReader, ReadError};
pub use writer::LineItemWriter;
