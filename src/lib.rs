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
    parts: Vec<String>,
}

fn is_space_or_comma(chr: char) -> bool {
    match chr {
        ' ' | ',' => true,
        _ => false
    }
}

fn part(text: &str) -> IResult<&str, &str> {
    let (text, _) = take_until("\t")(text)?;
    let (text, _) = tag("\t")(text)?;
    let (text, part) = take_until(";")(text)?;
    let (text, _) = tag(";\n")(text)?;
    return Ok((text, part))
}

fn inputs(text: &str) -> IResult<&str, &str> {
    let (text, _) = take_while(is_space_or_comma)(text)?;
    let (text, input) = take_till(is_space_or_comma)(text)?;
    let (text, _) = take_while(is_space_or_comma)(text)?;
    return Ok((text, input))
}

pub fn parse(text: &str) -> IResult<&str, Chip> {
    let (text, _) = tag("CHIP ")(text)?;
    let (text, chip_name) = take_until(" ")(text)?;
    let (text, _) = take_until("IN ")(text)?;
    let (text, _) = tag("IN ")(text)?;
    let (text, inputs) = take_until(";")(text)?;
    // let (Text, inputs) = many0(inputs)(text)?;
    let (text, _) = tag(";")(text)?;
    let (text, _) = take_until("OUT ")(text)?;
    let (text, _) = tag("OUT ")(text)?;
    let (text, outputs) = take_until(";")(text)?;
    let (text, _) = take_until("PARTS:\n")(text)?;
    let (text, _) = tag("PARTS:\n")(text)?;

    let (text, parts) = many0(part)(text)?;

    let inputs = inputs.replace(" ", "").split(",").map(|x| x.to_string()).collect::<Vec<String>>();
    let outputs = outputs.replace(" ", "").split(",").map(|x| x.to_string()).collect::<Vec<String>>();
    let parts = parts.iter().map(|x| x.to_string()).collect::<Vec<String>>();
    // println!("CHIPNAME: {}", chip_name);
    // println!("INPUTS: {:?}", inputs);
    // println!("OUTPUTS: {:?}", outputs);
    // println!("PARTS: {:?}", parts);
    return Ok((text, Chip {
        name: chip_name.to_string(),
        inputs: inputs,
        outputs: outputs,
        parts: parts,
    }));
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
