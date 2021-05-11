use crate::*;
use crate::parsing::parser::parse_script;
use std::fs;
use std::io::{Read, Seek, SeekFrom};

pub fn bind_mappings(script_file: &mut fs::File) -> Block {
    let script_file_length = script_file.seek(SeekFrom::End(0))
        .map_err(|err| anyhow!("failed to query script file length: {}", err)).unwrap();

    // restore head
    script_file.seek(SeekFrom::Start(0));

    let mut raw = String::with_capacity(script_file_length as usize);
    script_file.read_to_string(&mut raw);
    let mut global = parse_script(&*raw).unwrap();

    global
}
