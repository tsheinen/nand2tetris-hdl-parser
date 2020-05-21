//! test

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
/// This type use a `String` to store the pin name and `u32` to store both the start and end range of the pin
#[derive(Eq, PartialEq)]
pub struct Pin {
    name: String,
    start: u32,
    end: u32,
}

impl fmt::Debug for Pin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.start {
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


/// A type that represents a Chip
///
/// This type use a `String` to store the chip name, `Vec<Pin>` to store inputs and outputs, and `Vec<Part>` to store the parts that make up the chip
#[derive(Debug, Eq, PartialEq)]
pub struct Chip {
    name: String,
    inputs: Vec<Pin>,
    outputs: Vec<Pin>,
    parts: Vec<Part>,
}

/// A type that represents a Part
///
/// This type use a `String` to store the chip name and `Vec<Pin>` to store the connections from the main chip
#[derive(Debug, Eq, PartialEq)]
pub struct Part {
    name: String,
    internal: Vec<Pin>,
    external: Vec<Pin>,
}

fn internal_pin(text: &str) -> IResult<&str, (u32, u32)> {
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

fn part_pin(text: &str) -> IResult<&str, Pin> {
    let (text, _) = take_till(|x| is_alphabetic(x as u8))(text)?;
    let (text, name) = take_till(|x| !is_alphabetic(x as u8))(text)?;
    return match internal_pin(text) {
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

// TODO make this less awful
fn part(text: &str) -> IResult<&str, Part> {
    let (text, _) = take_till(|x| is_alphabetic(x as u8))(text)?;
    let (text, name) = take_until("(")(text)?;
    let (text, _) = tag("(")(text)?;
    let (text, pins) = take_until(")")(text)?;
    let pins_vec: (Vec<Pin>, Vec<Pin>) = pins
        .replace(" ", "")
        .split(',')
        .map(|x| x.split('=').collect::<Vec<&str>>())
        .map(|x| {
            let (_, lhs) = part_pin(x[0]).unwrap();
            let (_, rhs) = part_pin(x[1]).unwrap();
            (lhs, rhs)
        })
        .unzip();
    let (text, _) = tag(");")(text)?;
    let (text, _) = separator(text)?;
    Ok((text, Part {
        name: name.to_string(),
        internal: pins_vec.0,
        external: pins_vec.1,
    }))
}

fn interface_pin(text: &str) -> IResult<&str, Pin> {
    let (text, _) = not(tag(";"))(text)?;
    let (text, name) = take_till(|x| !is_alphabetic(x as u8))(text)?;
    let (text, (start, end)) = match internal_pin(text) {
        Ok((text, (start, end))) => (text, (start, end)),
        Err(_) => (text, (0, 0))
    };

    let (text, _) = opt(tag(","))(text)?;
    let (text, _) = separator(text)?;
    Ok((text, Pin {
        name: name.to_string(),
        start: start,
        end: end,
    }))
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
    Ok((many0(alt((|x| Ok((multispace1(x)?.0, ())), comment_line, comment_multiline)))(text)?.0,()))
}

fn parse_io_pins<'a>(text: &'a str, label: &str) -> IResult<&'a str, Vec<Pin>> {
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
}
