use clap::{App, Arg, SubCommand};
use anyhow::{Error, Result, anyhow};
use std::fs;

pub(super) struct Configuration {
    pub(super) script_file: fs::File,
    pub(super) verbose: bool,
}

pub(super) fn parse_cli() -> Result<Configuration> {
    let matches = App::new("key-mods")
        .version("1.0")
        .author("shiro <shiro@usagi.io>")
        .about("A scripting language that allows complex key remapping on Linux.")
        .arg(Arg::with_name("verbose")
            .short("-v")
            .long("--verbose")
            .help("Prints verbose information"))
        .arg(Arg::with_name("script file")
            .help("Executes the given script file")
            .index(1)
            .required(true))
        .get_matches();


    let script_path = matches.value_of("script file").unwrap().to_string();
    let script_file = fs::File::open(&script_path)
        .map_err(|err| anyhow!("failed to read script file '{}': {}", &script_path, &err))?;

    let config = Configuration {
        script_file,
        verbose: matches.is_present("verbose"),
    };

    Ok(config)
}