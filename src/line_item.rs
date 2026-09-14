use std::fmt;

/// One line of an invoice: what was sold, how much of it, and for how much.
///
/// Money is kept in integer cents rather than a float. Invoice totals get
/// compared and summed a lot, and float drift on a few thousand lines is
/// the kind of bug that only shows up when someone's books don't balance.
#[derive(Debug, Clone, PartialEq)]
pub struct LineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price_cents: i64,
    pub amount_cents: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    WrongFieldCount { expected: usize, found: usize },
    UnterminatedQuote,
    EmptyDescription,
    InvalidQuantity(String),
    InvalidUnitPrice(String),
    InvalidAmount(String),
    AmountMismatch { expected: i64, found: i64 },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::WrongFieldCount { expected, found } => {
                write!(f, "expected {} fields, found {}", expected, found)
            }
            ParseError::UnterminatedQuote => write!(f, "quoted field is missing its closing quote"),
            ParseError::EmptyDescription => write!(f, "description is empty"),
            ParseError::InvalidQuantity(s) => write!(f, "invalid quantity: {:?}", s),
            ParseError::InvalidUnitPrice(s) => write!(f, "invalid unit price: {:?}", s),
            ParseError::InvalidAmount(s) => write!(f, "invalid amount: {:?}", s),
            ParseError::AmountMismatch { expected, found } => write!(
                f,
                "amount_cents is {} but quantity * unit_price_cents is {}",
                found, expected
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// Splits a line into comma-separated fields, honoring `"..."` quoting.
///
/// A field can be wrapped in double quotes to contain a literal comma; a
/// literal quote inside a quoted field is written as `""`. Quoting only
/// takes effect if the quote is the first non-whitespace character of the
/// field, so `abc"def` is left alone rather than treated as malformed.
fn split_fields(line: &str) -> Result<Vec<String>, ParseError> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
        } else if c == '"' && field.trim().is_empty() {
            field.clear();
            in_quotes = true;
        } else if c == ',' {
            fields.push(std::mem::take(&mut field));
        } else {
            field.push(c);
        }
    }

    if in_quotes {
        return Err(ParseError::UnterminatedQuote);
    }
    fields.push(field);
    Ok(fields)
}

/// Wraps a field in double quotes and doubles any quotes inside it, if
/// that's needed for the field to round-trip through [`split_fields`].
///
/// Quoting is required when the field contains a comma, a quote, or a
/// line break, or when it starts with a quote - `split_fields` treats a
/// leading quote as the start of a quoted field regardless of intent, so
/// an unquoted field that happens to start with `"` would be misread.
fn quote_field_if_needed(field: &str) -> String {
    let needs_quoting =
        field.starts_with('"') || field.contains(['"', ',', '\n', '\r']);
    if !needs_quoting {
        return field.to_string();
    }
    let mut quoted = String::with_capacity(field.len() + 2);
    quoted.push('"');
    for c in field.chars() {
        if c == '"' {
            quoted.push('"');
        }
        quoted.push(c);
    }
    quoted.push('"');
    quoted
}

impl LineItem {
    /// Formats this line item back into the
    /// `description,quantity,unit_price_cents,amount_cents` line format,
    /// without a trailing newline. Quotes the description if it needs it
    /// to round-trip through [`LineItem::parse`].
    pub fn to_line(&self) -> String {
        format!(
            "{},{},{},{}",
            quote_field_if_needed(&self.description),
            self.quantity,
            self.unit_price_cents,
            self.amount_cents
        )
    }

    /// Parses a single line in the form
    /// `description,quantity,unit_price_cents,amount_cents`.
    ///
    /// A field may be wrapped in double quotes so it can contain a comma,
    /// e.g. `"widgets, deluxe",3,1250,3750`. A literal quote inside a
    /// quoted field is written as `""`.
    pub fn parse(line: &str) -> Result<LineItem, ParseError> {
        let fields = split_fields(line.trim_end_matches(['\r', '\n']))?;
        if fields.len() != 4 {
            return Err(ParseError::WrongFieldCount {
                expected: 4,
                found: fields.len(),
            });
        }

        let description = fields[0].trim();
        if description.is_empty() {
            return Err(ParseError::EmptyDescription);
        }

        let quantity: f64 = fields[1]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidQuantity(fields[1].clone()))?;
        let unit_price_cents: i64 = fields[2]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidUnitPrice(fields[2].clone()))?;
        let amount_cents: i64 = fields[3]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidAmount(fields[3].clone()))?;

        // Rounded rather than compared as floats: quantity carries a few
        // decimal places at most, so the product lands within a fraction of
        // a cent of the true value and rounding to the nearest cent is
        // exact for every quantity this format is meant to carry.
        let expected_amount_cents = (quantity * unit_price_cents as f64).round() as i64;
        if expected_amount_cents != amount_cents {
            return Err(ParseError::AmountMismatch {
                expected: expected_amount_cents,
                found: amount_cents,
            });
        }

        Ok(LineItem {
            description: description.to_string(),
            quantity,
            unit_price_cents,
            amount_cents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_line() {
        let item = LineItem::parse("widget, 3, 1250, 3750").unwrap();
        assert_eq!(item.description, "widget");
        assert_eq!(item.quantity, 3.0);
        assert_eq!(item.unit_price_cents, 1250);
        assert_eq!(item.amount_cents, 3750);
    }

    #[test]
    fn rejects_missing_fields() {
        let err = LineItem::parse("widget, 3, 1250").unwrap_err();
        assert_eq!(
            err,
            ParseError::WrongFieldCount {
                expected: 4,
                found: 3
            }
        );
    }

    #[test]
    fn rejects_empty_description() {
        let err = LineItem::parse(" , 3, 1250, 3750").unwrap_err();
        assert_eq!(err, ParseError::EmptyDescription);
    }

    #[test]
    fn parses_a_quoted_description_containing_commas() {
        let item = LineItem::parse("\"widgets, deluxe\", 3, 1250, 3750").unwrap();
        assert_eq!(item.description, "widgets, deluxe");
        assert_eq!(item.amount_cents, 3750);
    }

    #[test]
    fn unescapes_doubled_quotes_inside_a_quoted_field() {
        let item = LineItem::parse("\"6\"\" pipe\", 1, 500, 500").unwrap();
        assert_eq!(item.description, "6\" pipe");
    }

    #[test]
    fn rejects_an_unterminated_quote() {
        let err = LineItem::parse("\"widgets, 3, 1250, 3750").unwrap_err();
        assert_eq!(err, ParseError::UnterminatedQuote);
    }

    #[test]
    fn leaves_an_embedded_quote_alone_when_not_at_field_start() {
        let item = LineItem::parse("6\" pipe, 1, 500, 500").unwrap();
        assert_eq!(item.description, "6\" pipe");
    }

    #[test]
    fn rejects_an_amount_that_does_not_match_quantity_times_unit_price() {
        let err = LineItem::parse("widget, 3, 1250, 4000").unwrap_err();
        assert_eq!(
            err,
            ParseError::AmountMismatch {
                expected: 3750,
                found: 4000
            }
        );
    }

    #[test]
    fn accepts_an_amount_that_rounds_to_the_nearest_cent() {
        let item = LineItem::parse("bulk item, 0.1, 3, 0").unwrap();
        assert_eq!(item.amount_cents, 0);
    }

    #[test]
    fn formats_a_plain_line_without_quoting() {
        let item = LineItem::parse("widget, 3, 1250, 3750").unwrap();
        assert_eq!(item.to_line(), "widget,3,1250,3750");
    }

    #[test]
    fn quotes_a_description_containing_a_comma_when_formatting() {
        let item = LineItem::parse("\"widgets, deluxe\", 3, 1250, 3750").unwrap();
        assert_eq!(item.to_line(), "\"widgets, deluxe\",3,1250,3750");
    }

    #[test]
    fn doubles_quotes_inside_a_quoted_description_when_formatting() {
        let item = LineItem::parse("\"6\"\" pipe\", 1, 500, 500").unwrap();
        assert_eq!(item.to_line(), "\"6\"\" pipe\",1,500,500");
    }

    #[test]
    fn quotes_a_description_that_starts_with_a_quote_even_without_a_comma() {
        let item = LineItem {
            description: "\"inch mark".to_string(),
            quantity: 1.0,
            unit_price_cents: 500,
            amount_cents: 500,
        };
        assert_eq!(item.to_line(), "\"\"\"inch mark\",1,500,500");
    }

    #[test]
    fn round_trips_through_parse_and_to_line() {
        let original = LineItem::parse("\"widgets, deluxe\", 3.5, 1250, 4375").unwrap();
        let formatted = original.to_line();
        let reparsed = LineItem::parse(&formatted).unwrap();
        assert_eq!(original, reparsed);
    }
}
