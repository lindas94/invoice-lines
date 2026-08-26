# invoice-lines

A small Rust library for reading invoice line item exports.

The exports I deal with come out of billing systems as flat files: one line
item per row, tens or hundreds of thousands of rows for a month's worth of
invoices. Loading a file like that into a `Vec<LineItem>` before you can
even start looking at it wastes memory for no reason - you usually just
want to walk the rows once, check them, and sum a few totals. This library
reads one line at a time and never holds more than that in memory,
regardless of how big the input is.

No third-party dependencies. Standard library only.

## Line format

Each line is:

```
description,quantity,unit_price_cents,amount_cents
```

- `description` - free text, currently must not contain a comma (see
  "Limitations" below)
- `quantity` - decimal, e.g. `2.5`
- `unit_price_cents` and `amount_cents` - integers, in cents, to avoid
  float rounding when totals get summed

Example:

```
16GB RAM upgrade,1,4500,4500
consulting hours,3.5,12000,42000
```

Blank lines are skipped. Anything else that doesn't fit the shape above
produces a `ParseError` that names the line number and what was wrong
with it.

## Usage

```rust
use invoice_lines::LineItemReader;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("invoice_export.csv")?;
    let reader = LineItemReader::new(file);

    let mut total_cents: i64 = 0;
    for result in reader {
        let item = result?;
        total_cents += item.amount_cents;
    }

    println!("total: {}.{:02}", total_cents / 100, total_cents % 100);
    Ok(())
}
```

`LineItemReader` works over anything implementing `std::io::Read`, so it
also reads directly from a socket or a pipe without changes:

```rust
use invoice_lines::LineItemReader;
use std::io::stdin;

fn main() {
    for result in LineItemReader::new(stdin()) {
        match result {
            Ok(item) => println!("{}: {}", item.description, item.amount_cents),
            Err(e) => eprintln!("skipping bad row: {}", e),
        }
    }
}
```

## Limitations

This is an early skeleton. Known gaps:

- No support for quoted fields, so a description containing a comma will
  misparse. Real invoice exports often quote free text, so this needs
  fixing before the library is useful on real data.
- `amount_cents` is taken as given from the input; nothing currently
  checks it against `quantity * unit_price_cents`.
- No writer/serializer yet, only reading.

## License

MIT, see LICENSE.
