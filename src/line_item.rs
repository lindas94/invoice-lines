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
    EmptyDescription,
    InvalidQuantity(String),
    InvalidUnitPrice(String),
    InvalidAmount(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::WrongFieldCount { expected, found } => {
                write!(f, "expected {} fields, found {}", expected, found)
            }
            ParseError::EmptyDescription => write!(f, "description is empty"),
            ParseError::InvalidQuantity(s) => write!(f, "invalid quantity: {:?}", s),
            ParseError::InvalidUnitPrice(s) => write!(f, "invalid unit price: {:?}", s),
            ParseError::InvalidAmount(s) => write!(f, "invalid amount: {:?}", s),
        }
    }
}

impl std::error::Error for ParseError {}

impl LineItem {
    /// Parses a single line in the form
    /// `description,quantity,unit_price_cents,amount_cents`.
    ///
    /// Descriptions can't contain commas yet - quoting isn't implemented
    /// in this version, so a comma in the description will misalign the
    /// remaining fields and fail with `WrongFieldCount`.
    pub fn parse(line: &str) -> Result<LineItem, ParseError> {
        let fields: Vec<&str> = line.trim_end_matches(['\r', '\n']).split(',').collect();
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
            .map_err(|_| ParseError::InvalidQuantity(fields[1].to_string()))?;
        let unit_price_cents: i64 = fields[2]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidUnitPrice(fields[2].to_string()))?;
        let amount_cents: i64 = fields[3]
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidAmount(fields[3].to_string()))?;

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
}
