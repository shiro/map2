use crate::*;
use crate::parsing::parser::parse_script;
use std::fs;
use std::io::{Read, Seek};

pub fn bind_mappings(script_file: &mut fs::File) -> Block {
    let mut raw = String::with_capacity(script_file.stream_len().unwrap() as usize);
    script_file.read_to_string(&mut raw);
    let mut global = parse_script(&*raw).unwrap();

    global
}
