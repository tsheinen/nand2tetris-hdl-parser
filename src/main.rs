use std::{fs, env};
use hack_hdl::{Chip, parse_hdl};

fn main() {
    let hdl = match fs::read_to_string(env::args().nth(1).unwrap()) {
        Ok(f) => f,
        Err(e) => panic!("Could not read file {:?}: {:?}", env::args().nth(1).unwrap(), e),
    };

    println!("{:#?}", match parse_hdl(&hdl) {
        Ok(x) => x.1,
        Err(e) => panic!("{}",e),
    });
}