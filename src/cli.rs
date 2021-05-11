use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{App, Arg, };
use xdg::BaseDirectories;

pub struct Configuration {
    pub script_file: fs::File,
    pub verbose: bool,
    pub devices: Vec<String>,
}

pub fn parse_cli() -> Result<Configuration> {
    let matches = App::new("map2")
        .version("1.0")
        .author("shiro <shiro@usagi.io>")
        .about("A scripting language that allows complex key remapping on Linux.")
        .arg(Arg::with_name("verbose")
            .short("-v")
            .long("--verbose")
            .help("Prints verbose information"))
        .arg(Arg::with_name("devices")
            .help("Selects the input devices")
            .short("-d")
            .long("--devices")
            .takes_value(true)
        )
        .arg(Arg::with_name("script file")
            .help("Executes the given script file")
            .index(1)
            .required(true))
        .get_matches();

    let device_list_config_name = "devices.list";

    let xdg_dirs = BaseDirectories::with_prefix("map2")
        .map_err(|_| anyhow!("failed to initialize XDG directory configuration"))?;

    let script_path = matches.value_of("script file").unwrap().to_string();
    let script_file = fs::File::open(&script_path)
        .map_err(|err| anyhow!("failed to read script file '{}': {}", &script_path, &err))?;

    let device_list_path = matches.value_of("devices")
        .map(|path| Some(PathBuf::from(path)))
        .unwrap_or_else(|| {
            xdg_dirs.find_config_file(&device_list_config_name)
        })
        .map(|path| {
            fs::File::open(PathBuf::from(&path))
                .map_err(|err| anyhow!("failed to open device list '{}': {}", &path.display(), err))
        });

    let device_list = match device_list_path {
        Some(path) => BufReader::new(path?)
            .lines()
            .collect::<std::result::Result<_, _>>()
            .map_err(|err| anyhow!("failed to parse devices file: {}", err))?,
        None => { vec![] }
    };

    let config = Configuration {
        script_file,
        verbose: matches.is_present("verbose"),
        devices: device_list,
    };

    Ok(config)
}