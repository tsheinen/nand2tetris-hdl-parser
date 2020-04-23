#[macro_use]
extern crate nom;

use nom::IResult;
use nom::bytes::complete::{tag, take_while_m_n, take_until, take_till, take_while};
use nom::sequence::delimited;
use nom::multi::{many0, many1};
use nom::lib::std::fmt::Error;

#[derive(Debug)]
pub struct Chip {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    parts: Vec<Part>,
}

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
    let pins2 = pins
        .replace(" ", "")
        .split(",")
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let mut part = Part {
        name: name.to_string(),
        internal: vec![],
        external: vec![],
    };
    for i in pins2 {
        let a = i.split("=").collect::<Vec<&str>>();
        part.internal.push(a[0].to_string());
        part.external.push(a[1].to_string());
    }
    return Ok((text, part))
}

fn inputs(text: &str) -> IResult<&str, Vec<String>> {
    let (text, _) = tag("IN ")(text)?;
    let (text, inputs) = take_until(";")(text)?;
    return Ok((text, inputs.replace(" ", "").split(",").map(|x| x.to_string()).collect::<Vec<String>>()))
}

fn outputs(text: &str) -> IResult<&str, Vec<String>> {
    let (text, _) = tag("OUT ")(text)?;
    let (text, inputs) = take_until(";")(text)?;
    return Ok((text, inputs.replace(" ", "").split(",").map(|x| x.to_string()).collect::<Vec<String>>()))
}

fn parts(text: &str) -> IResult<&str, Vec<Part>> {
    let (text, _) = tag("PARTS:\n")(text)?;
    let (text, parts) = many0(part)(text)?;
    return Ok((text, parts))
}

pub fn parse_hdl(text: &str) -> IResult<&str, Chip> {
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
