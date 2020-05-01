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

use nom::IResult;
use nom::bytes::complete::{tag, take_until};
use nom::multi::many0;


/// Represents a pin
#[derive(Debug)]
pub struct Pin {
    name: String,
    start: u32,
    end: u32,
}

/// Represents a parsed chip
#[derive(Debug)]
pub struct Chip {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    parts: Vec<Part>,
}

/// Represents an internal chip
#[derive(Debug)]
pub struct Part {
    name: String,
    internal: Vec<String>,
    external: Vec<String>,
}

// TODO make this less awful
fn part(text: &str) -> IResult<&str, Part> {
    let (text, _) = take_until("\t")(text)?;
    let (text, _) = tag("\t")(text)?;
    let (text, name) = take_until("(")(text)?;
    let (text, _) = tag("(")(text)?;
    let (text, pins) = take_until(")")(text)?;
    let pins_vec: (Vec<String>, Vec<String>) = pins
        .replace(" ", "")
        .split(',')
        .map(|x| x.split('=').collect::<Vec<&str>>())
        .map(|x| (x[0].to_string(), x[1].to_string()))
        .unzip();
    Ok((text, Part {
        name: name.to_string(),
        internal: pins_vec.0,
        external: pins_vec.1,
    }))
}

fn inputs(text: &str) -> IResult<&str, Vec<String>> {
    let (text, _) = tag("IN ")(text)?;
    let (text, inputs) = take_until(";")(text)?;
    return Ok((text, inputs.replace(" ", "").split(',').map(|x| x.to_string()).collect::<Vec<String>>()))
}

fn outputs(text: &str) -> IResult<&str, Vec<String>> {
    let (text, _) = tag("OUT ")(text)?;
    let (text, inputs) = take_until(";")(text)?;
    return Ok((text, inputs.replace(" ", "").split(',').map(|x| x.to_string()).collect::<Vec<String>>()))
}

fn parts(text: &str) -> IResult<&str, Vec<Part>> {
    let (text, _) = tag("PARTS:\n")(text)?;
    let (text, parts) = many0(part)(text)?;
    return Ok((text, parts))
}


/// parse a string in nand2tetris hack hdl into a Chip struct
pub fn parse_hdl(text: &str) -> IResult<&str, Chip> {
    // let comments_regex = Regex::new("//.*$").unwrap();
    // let text = comments_regex.replace(text, "").into_owned().as_str();
    let (text, _) = tag("CHIP ")(text)?;
    let (text, chip_name) = take_until(" ")(text)?;

    let (text, _) = take_until("IN ")(text)?;
    let (text, inputs) = inputs(text)?;

    let (text, _) = take_until("OUT ")(text)?;
    let (text, outputs) = outputs(text)?;

    let (text, _) = take_until("PARTS:\n")(text)?;
    let (text, parts) = parts(text)?;

    return Ok((text, Chip {
        name: chip_name.to_string(),
        inputs,
        outputs,
        parts,
    }));
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
