use crate::*;
use crate::parsing::parser::parse_script;

pub(crate) fn bind_mappings() -> Block {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { panic!("no script file arg given"); }

    let script_filename = &args[1];

    let script = std::fs::read_to_string(script_filename).expect("failed to read file");

    // let script = "a::b;  b::c;".to_string();
    let mut global = parse_script(script.as_str()).unwrap();

    global
}
