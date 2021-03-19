//! parser for nand2tetris HDL

#![forbid(unsafe_code)]
#![deny(
missing_debug_implementations,
missing_docs,
trivial_casts,
trivial_numeric_casts,
unused_extern_crates,
unused_import_braces,
unused_qualifications,
unused_results,
warnings
)]

mod python;

use core::fmt;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until, take_while1};
use nom::character::complete::{line_ending, multispace1};
use nom::character::streaming::digit1;
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::combinator::{not, opt};
use nom::error::{convert_error, VerboseError, context};
use nom::multi::{many0, many1};
use nom::{Err, IResult};
use std::error::Error;
use serde::{Deserialize, Serialize};
use dict_derive::{FromPyObject, IntoPyObject};

/// A type that represents a pin
///
/**
```rust
pub struct Pin {
    pub name: String,
    pub start: u32,
    pub end: u32,
}
```
*/
#[derive(Eq, PartialEq, Hash, Clone, Serialize, Deserialize, FromPyObject, IntoPyObject)]
pub struct Pin {
    /// Holds the name of the pin
    pub name: String,
    /// Holds the start of the slice range
    pub start: u32,
    /// Holds the end of the slice range
    pub end: u32,
}

impl fmt::Debug for Pin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            f.debug_struct("Pin")
                .field("name", &self.name)
                .field("index", &self.start)
                .finish()
        } else {
            f.debug_struct("Pin")
                .field("name", &self.name)
                .field("start", &self.start)
                .field("end", &self.end)
                .finish()
        }
    }
}

/// A type that represents a chip
///
/**
```rust
use nand2tetris_hdl_parser::{Part, Pin};
pub struct Chip {
    pub name: String,
    pub inputs: Vec<Pin>,
    pub outputs: Vec<Pin>,
    pub parts: Vec<Part>,
}
```
*/
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, FromPyObject, IntoPyObject)]
pub struct Chip {
    /// Holds the name of the chip
    pub name: String,
    /// Holds a list of input pins
    pub inputs: Vec<Pin>,
    /// Holds a list of output pins
    pub outputs: Vec<Pin>,
    /// Holds a list of parts
    pub parts: Vec<Part>,
}

/// A type that represents a part
/// Internal pins are pins that match up to an input/output of the part - the first pin in a {}={} pair
/// Internal pins are pins that match up to another part of the chip - the second pin in a {}={} pair
///
/**
```rust
use nand2tetris_hdl_parser::Pin;
pub struct Part {
    pub name: String,
    pub internal: Vec<Pin>,
    pub external: Vec<Pin>,
}
```
*/
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, FromPyObject, IntoPyObject)]
pub struct Part {
    /// Holds the name of the part
    pub name: String,
    /// Holds internal connections (the pins which match up to the input pins of the part)
    pub internal: Vec<Pin>,
    /// Holds external connections
    pub external: Vec<Pin>,
}



/// Error returned when HDL cannot be parsed
#[derive(Debug)]
pub struct HDLParseError {
    details: String,
}

impl HDLParseError {
    fn new(msg: &str) -> HDLParseError {
        HDLParseError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for HDLParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for HDLParseError {
    fn description(&self) -> &str {
        &self.details
    }
}

/// Try to consume whitespace, line comments, and multiline comments until all three fail on the same text
/// Matches 0 or more
fn separator(text: &str) -> IResult<&str, (), VerboseError<&str>> {
    fn comment_line(text: &str) -> IResult<&str, (), VerboseError<&str>> {
        let (text, _) = tag("//")(text)?;
        let (text, _) = take_until("\n")(text)?;
        let (text, _) = line_ending(text)?;
        Ok((text, ()))
    }
    fn comment_multiline(text: &str) -> IResult<&str, (), VerboseError<&str>> {
        let (text, _) = tag("/**")(text)?;
        let (text, _) = take_until("*/")(text)?;
        let (text, _) = tag("*/")(text)?;

        Ok((text, ()))
    }
    Ok((
        many0(alt((
            |x| Ok((multispace1(x)?.0, ())),
            comment_line,
            comment_multiline,
        )))(text)?
            .0,
        (),
    ))
}

/// Parses a pin descriptor into a [Pin]
///
/// `a[0..3]` will become Pin { name: "a", start: 0, end: 3 }
fn pin(text: &str) -> IResult<&str, Pin, VerboseError<&str>> {
    /// parses a pin range descriptor into `(u32, u32)`.  Both u32 will be the same if the range is a single number.
    ///
    /// `[0..3]` will parse into (0, 3) and `[0]` will parse into (0,0)
    fn pin_index(text: &str) -> IResult<&str, (u32, u32), VerboseError<&str>> {
        fn internal_pin_single(text: &str) -> IResult<&str, (u32, u32), VerboseError<&str>> {
            let (text, _) = tag("[")(text)?;
            let (text, index) = digit1(text)?;
            let (text, _) = tag("]")(text)?;

            Ok((
                text,
                (
                    index.parse::<u32>().unwrap_or(0),
                    index.parse::<u32>().unwrap_or(0),
                ),
            ))
        }
        fn internal_pin_range(text: &str) -> IResult<&str, (u32, u32), VerboseError<&str>> {
            let (text, _) = tag("[")(text)?;
            let (text, start) = digit1(text)?;
            let (text, _) = tag("..")(text)?;
            let (text, end) = digit1(text)?;
            let (text, _) = tag("]")(text)?;

            Ok((
                text,
                (
                    start.parse::<u32>().unwrap_or(0),
                    end.parse::<u32>().unwrap_or(0),
                ),
            ))
        }
        alt((internal_pin_single, internal_pin_range))(text)
    }

    let (text, _) = take_till(|x| is_alphabetic(x as u8))(text)?;
    let (text, name) = take_till(|x| match x {
        ',' | ')' | ';' | '=' | '[' | ' ' => true,
        _ => false,
    })(text)?;
    let (text, _) = separator(text)?;
    match pin_index(text) {
        Ok((text, (start, end))) => Ok((
            text,
            Pin {
                name: name.to_string(),
                start,
                end,
            },
        )),
        Err(_) => {
            Ok((
                text,
                Pin {
                    name: name.to_string(),
                    start: 0,
                    end: 0,
                },
            ))
        }
    }
}

/// Parses a part descriptor into a [Part]
///
/// `Test(a[0..3]=a[0..3],b=b,out=out);` will become a part with the name `Test` and the pins parsed with [part_pin]
fn part(text: &str) -> IResult<&str, Part, VerboseError<&str>> {
    fn internal_part(text: &str) -> IResult<&str, (Pin, Pin), VerboseError<&str>> {
        let (text, _) = not(tag(")"))(text)?;
        let (text, pin1) = pin(text)?;
        let (text, _) = tag("=")(text)?;
        let (text, pin2) = pin(text)?;
        Ok((text, (pin1, pin2)))
    }

    let (text, _) = separator(text)?;
    let (text, name) = context("expected nonzero length alphanumeric identifier", take_while1(|x| is_alphanumeric(x as u8)))(text)?;
    let (text, _) = separator(text)?;
    let (text, _) = context("symbol \"(\"", tag("("))(text)?;
    let (text, pins) = many0(internal_part)(text)?;
    let pins = pins
        .iter()
        .map(|&(ref a, ref b)| (a.clone(), b.clone()))
        .unzip();

    let (text, _) = context("symbol \");\"", tag(");"))(text)?;
    let (text, _) = separator(text)?;
    Ok((
        text,
        Part {
            name: name.to_string(),
            internal: pins.0,
            external: pins.1,
        },
    ))
}

/// parse input/output pin line with arbitrary label
///
/// `IN a, b;` would parse into a `Vec<Pin>` with two pins - a and b
fn parse_io_pins<'a>(
    text: &'a str,
    label: &'static str,
) -> IResult<&'a str, Vec<Pin>, VerboseError<&'a str>> {
    fn interface_pin(text: &str) -> IResult<&str, Pin, VerboseError<&str>> {
        let (text, _) = not(tag(";"))(text)?;
        let (text, pin) = pin(text)?;

        let (text, _) = opt(tag(","))(text)?;
        let (text, _) = separator(text)?;
        Ok((text, pin))
    }

    let (text, _) = separator(text)?;
    let (text, _) = take_until(label)(text)?;

    let (text, _) = separator(text)?;
    let (text, _) = context(label, tag(label))(text)?;
    let (text, _) = separator(text)?;

    let (text, inputs) = many1(context("Pin", interface_pin))(text)?;

    let (text, _) = separator(text)?;
    Ok((text, inputs))
}

/// parse_hdl will consume text and return `Result<Chip, Error>` depending on if it can successfully be parsed
pub fn parse_hdl(text: &str) -> Result<Chip, HDLParseError> {
    fn parse_hdl_internal(text: &str) -> IResult<&str, Chip, VerboseError<&str>> {
        let (text, _) = separator(text)?;
        let (text, _) = context("symbol \"CHIP\"", tag("CHIP"))(text)?;

        let (text, _) = separator(text)?;
        let (text, chip_name) = context("alphanumeric identifier (for name)", take_till(|x| !is_alphanumeric(x as u8)))(text)?;

        let (text, inputs) = parse_io_pins(text, "IN")?;
        let (text, outputs) = parse_io_pins(text, "OUT")?;

        let (text, _) = take_until("PARTS:")(text)?;
        let (text, _) = context("symbol \"PARTS:\"", tag("PARTS:"))(text)?;
        let (text, _) = separator(text)?;
        let (text, parts) = many0(part)(text)?;

        Ok((
            text,
            Chip {
                name: chip_name.to_string(),
                inputs,
                outputs,
                parts,
            },
        ))
    }

    match parse_hdl_internal(text) {
        Ok((_, chip)) => Ok(chip),
        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            Err(HDLParseError::new(&convert_error(text, e)))
        }
        _ => Err(HDLParseError::new(
            "Should never happen, report it if it does",
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_hdl, parse_io_pins, Chip, Part, Pin};
    use std::fs;
    use std::io::Error;

    #[test]
    fn fails_parse() -> Result<(), Error> {
        assert_eq!(
            format!("{}", parse_hdl("aaaa ").err().unwrap()),
            "0: at line 1, in Tag:
aaaa
^

1: at line 1, in symbol \"CHIP\":
aaaa
^

"
        );
        Ok(())
    }

    #[test]
    fn example_hdl() -> Result<(), Error> {
        let example_hdl_text = fs::read_to_string("test_cases/example.hdl")?;
        let example_hdl_chip = Chip {
            name: "Example".to_string(),
            inputs: vec![
                Pin {
                    name: "a".to_string(),
                    start: 0,
                    end: 0,
                },
                Pin {
                    name: "b".to_string(),
                    start: 0,
                    end: 0,
                },
            ],
            outputs: vec![Pin {
                name: "out".to_string(),
                start: 0,
                end: 0,
            }],
            parts: vec![
                Part {
                    name: "Test".to_string(),
                    internal: vec![
                        Pin {
                            name: "a".to_string(),
                            start: 0,
                            end: 3,
                        },
                        Pin {
                            name: "b".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "out".to_string(),
                            start: 0,
                            end: 0,
                        },
                    ],
                    external: vec![
                        Pin {
                            name: "a".to_string(),
                            start: 0,
                            end: 3,
                        },
                        Pin {
                            name: "b".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "out1".to_string(),
                            start: 0,
                            end: 0,
                        },
                    ],
                },
                Part {
                    name: "Test".to_string(),
                    internal: vec![
                        Pin {
                            name: "a".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "b".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "out".to_string(),
                            start: 0,
                            end: 0,
                        },
                    ],
                    external: vec![
                        Pin {
                            name: "a".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "b".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "out".to_string(),
                            start: 0,
                            end: 0,
                        },
                    ],
                },
                Part {
                    name: "Test".to_string(),
                    internal: vec![
                        Pin {
                            name: "a".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "b".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "out".to_string(),
                            start: 0,
                            end: 0,
                        },
                    ],
                    external: vec![
                        Pin {
                            name: "a".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "b".to_string(),
                            start: 0,
                            end: 0,
                        },
                        Pin {
                            name: "out".to_string(),
                            start: 0,
                            end: 0,
                        },
                    ],
                },
            ],
        };
        assert_eq!(example_hdl_chip, parse_hdl(&example_hdl_text).unwrap());
        Ok(())
    }

    #[test]
    fn test_parse_io_pins() -> Result<(), Error> {
        let text = "    IN a, b;
";
        let (_, pins) = parse_io_pins(text, "IN").unwrap_or(("", vec![]));
        assert_eq!(
            pins,
            vec![
                Pin {
                    name: "a".to_string(),
                    start: 0,
                    end: 0,
                },
                Pin {
                    name: "b".to_string(),
                    start: 0,
                    end: 0,
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn test_pin_debug_display() -> Result<(), Error> {
        let index_same_formatted: String = format!(
            "{:?}",
            Pin {
                name: "placeholder".to_string(),
                start: 0,
                end: 0,
            }
        )
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let index_same_cmp: String = "Pin { name: \"placeholder\", index: 0 }"
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        assert_eq!(index_same_formatted, index_same_cmp);

        let index_different_formatted: String = format!(
            "{:?}",
            Pin {
                name: "placeholder".to_string(),
                start: 3,
                end: 4,
            }
        )
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let index_different_cmp: String = "Pin { name: \"placeholder\", start: 3, end: 4 }"
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        assert_eq!(index_different_formatted, index_different_cmp);

        Ok(())
    }
}
