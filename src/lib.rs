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


use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::character::complete::{line_ending, multispace1};
use nom::character::streaming::digit1;
use nom::combinator::{not, opt};
use nom::IResult;
use nom::multi::{many0, many1};
use simple_error::SimpleError;
use core::fmt;

/// A type that represents a pin
///
/**
```rust
pub struct Pin {
    name: String,
    start: u32,
    end: u32,
}
```
*/
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Pin {
    name: String,
    start: u32,
    end: u32,
}

impl fmt::Debug for Pin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            f.debug_struct("Pin").field("name", &self.name).field("index", &self.start).finish()
        } else {
            f.debug_struct("Pin").field("name", &self.name).field("start", &self.start).field("end", &self.end).finish()
        }
    }
}


/// A type that represents a chip
///
/**
```rust
use nand2tetris_hdl_parser::{Part, Pin};
pub struct Chip {
    name: String,
    inputs: Vec<Pin>,
    outputs: Vec<Pin>,
    parts: Vec<Part>,
}
```
*/
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Chip {
    name: String,
    inputs: Vec<Pin>,
    outputs: Vec<Pin>,
    parts: Vec<Part>,
}

/// A type that represents a part
///
/**
```rust
use nand2tetris_hdl_parser::Pin;
pub struct Part {
    name: String,
    internal: Vec<Pin>,
    external: Vec<Pin>,
}
```
*/
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Part {
    name: String,
    internal: Vec<Pin>,
    external: Vec<Pin>,
}


/// Try to consume whitespace, line comments, and multiline comments until all three fail on the same text
/// Matches 0 or more
fn separator(text: &str) -> IResult<&str, ()> {
    fn comment_line(text: &str) -> IResult<&str, ()> {
        let (text, _) = tag("//")(text)?;
        let (text, _) = take_until("\n")(text)?;
        let (text, _) = line_ending(text)?;
        Ok((text, ()))
    }
    fn comment_multiline(text: &str) -> IResult<&str, ()> {
        let (text, _) = tag("/**")(text)?;
        let (text, _) = take_until("*/")(text)?;
        let (text, _) = tag("*/")(text)?;

        Ok((text, ()))
    }
    Ok((many0(alt((|x| Ok((multispace1(x)?.0, ())), comment_line, comment_multiline)))(text)?.0, ()))
}

/// Parses a pin descriptor into a [Pin]
///
/// `a[0..3]` will become Pin { name: "a", start: 0, end: 3 }
fn pin(text: &str) -> IResult<&str, Pin> {
    /// parses a pin range descriptor into `(u32, u32)`.  Both u32 will be the same if the range is a single number.
    ///
    /// `[0..3]` will parse into (0, 3) and `[0]` will parse into (0,0)
    fn pin_index(text: &str) -> IResult<&str, (u32, u32)> {
        fn internal_pin_single(text: &str) -> IResult<&str, (u32, u32)> {
            let (text, _) = tag("[")(text)?;
            let (text, index) = digit1(text)?;
            let (text, _) = tag("]")(text)?;

            Ok((text, (index.parse::<u32>().unwrap(), index.parse::<u32>().unwrap())))
        }
        fn internal_pin_range(text: &str) -> IResult<&str, (u32, u32)> {
            let (text, _) = tag("[")(text)?;
            let (text, start) = digit1(text)?;
            let (text, _) = tag("..")(text)?;
            let (text, end) = digit1(text)?;
            let (text, _) = tag("]")(text)?;

            Ok((text, (start.parse::<u32>().unwrap(), end.parse::<u32>().unwrap())))
        }
        alt((internal_pin_single, internal_pin_range))(text)
    }

    let (text, _) = take_till(|x| is_alphabetic(x as u8))(text)?;
    let (text, name) = take_till(|x| !is_alphabetic(x as u8))(text)?;
    return match pin_index(text) {
        Ok((text, (start, end))) => Ok((text, Pin {
            name: name.to_string(),
            start,
            end,
        })),
        Err(_) => return Ok((text, Pin {
            name: name.to_string(),
            start: 0,
            end: 0,
        }))
    };
}


/// Parses a part descriptor into a [Part]
///
/// `Test(a[0..3]=a[0..3],b=b,out=out);` will become a part with the name `Test` and the pins parsed with [part_pin]
fn part(text: &str) -> IResult<&str, Part> {
    fn internal_part(text: &str) -> IResult<&str, (Pin, Pin)> {
        let (text, _) = not(tag(")"))(text)?;
        let (text, pin1) = pin(text)?;
        let (text, _) = tag("=")(text)?;
        let (text, pin2) = pin(text)?;
        Ok((text, (pin1, pin2)))
    }
    let (text, _) = take_till(|x| is_alphabetic(x as u8))(text)?;
    let (text, name) = take_until("(")(text)?;
    let (text, _) = tag("(")(text)?;
    let (text, pins) = many0(internal_part)(text)?;
    let pins = pins
        .iter()
        .map(|&(ref a, ref b)| (a.clone(), b.clone()))
        .unzip();

    let (text, _) = tag(");")(text)?;
    let (text, _) = separator(text)?;
    Ok((text, Part {
        name: name.to_string(),
        internal: pins.0,
        external: pins.1,
    }))
}

/// parse input/output pin line with arbitrary label
///
/// `IN a, b;` would parse into a `Vec<Pin>` with two pins - a and b
fn parse_io_pins<'a>(text: &'a str, label: &str) -> IResult<&'a str, Vec<Pin>> {
    fn interface_pin(text: &str) -> IResult<&str, Pin> {
        let (text, _) = not(tag(";"))(text)?;
        let (text, pin) = pin(text)?;

        let (text, _) = opt(tag(","))(text)?;
        let (text, _) = separator(text)?;
        Ok((text, pin))
    }

    let (text, _) = separator(text)?;
    let (text, _) = take_until(label)(text)?;

    let (text, _) = separator(text)?;
    let (text, _) = tag(label)(text)?;
    let (text, _) = separator(text)?;

    let (text, inputs) = many1(interface_pin)(text)?;

    let (text, _) = separator(text)?;
    Ok((text, inputs))
}


/// parse_hdl will consume text and return `Result<Chip, Error>` depending on if it can successfully be parsed
pub fn parse_hdl(text: &str) -> Result<Chip, SimpleError> {
    fn parse_hdl_internal(text: &str) -> IResult<&str, Chip> {
        let (text, _) = separator(text)?;
        let (text, _) = tag("CHIP")(text)?;

        let (text, _) = separator(text)?;
        let (text, chip_name) = take_till(|x| !is_alphanumeric(x as u8))(text)?;

        let (text, inputs) = parse_io_pins(text, "IN")?;
        let (text, outputs) = parse_io_pins(text, "OUT")?;


        let (text, _) = take_until("PARTS:")(text)?;
        let (text, _) = tag("PARTS:")(text)?;
        let (text, _) = separator(text)?;
        let (text, parts) = many0(part)(text)?;

        return Ok((text, Chip {
            name: chip_name.to_string(),
            inputs,
            outputs,
            parts,
        }));
    }

    match parse_hdl_internal(text) {
        Ok((_, chip)) => Ok(chip),
        Err(_) => Err(SimpleError::new("Failed to parse"))
    }
}


#[cfg(test)]
mod tests {
    use crate::{Chip, Pin, Part, parse_hdl, parse_io_pins};
    use std::fs;
    use std::io::Error;

    #[test]
    fn fails_parse() -> Result<(), Error> {
        assert_eq!(parse_hdl("").err().unwrap().as_str(), "Failed to parse");
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
            outputs: vec![
                Pin {
                    name: "out".to_string(),
                    start: 0,
                    end: 0,
                },
            ],
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
        assert_eq!(pins, vec![
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
        ]);
        Ok(())
    }

    #[test]
    fn test_pin_debug_display() -> Result<(), Error> {
        let index_same_formatted: String = format!("{:?}", Pin {
            name: "placeholder".to_string(),
            start: 0,
            end: 0,
        }).chars().filter(|c| !c.is_whitespace()).collect();
        let index_same_cmp: String = "Pin { name: \"placeholder\", index: 0 }".chars().filter(|c| !c.is_whitespace()).collect();
        assert_eq!(index_same_formatted, index_same_cmp);

        let index_different_formatted: String = format!("{:?}", Pin {
            name: "placeholder".to_string(),
            start: 3,
            end: 4,
        }).chars().filter(|c| !c.is_whitespace()).collect();
        let index_different_cmp: String = "Pin { name: \"placeholder\", start: 3, end: 4 }".chars().filter(|c| !c.is_whitespace()).collect();
        assert_eq!(index_different_formatted, index_different_cmp);

        Ok(())
    }
}
