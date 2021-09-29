use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Arg};
use xdg::BaseDirectories;

pub struct Configuration {
    // pub script_file: fs::File,
    // pub verbosity: i32,
    pub devices: Vec<String>,
}

pub fn parse_cli() -> Result<Configuration> {
    // let matches = App::new("map2")
    //     .version("1.0")
    //     .author("shiro <shiro@usagi.io>")
    //     .about("A scripting language that allows complex key remapping on Linux.")
    //     .arg(Arg::with_name("verbosity")
    //         .short("-v")
    //         .long("--verbose")
    //         .multiple(true)
    //         .help("Sets the verbosity level"))
    //     .arg(Arg::with_name("devices")
    //         .help("Selects the input devices")
    //         .short("-d")
    //         .long("--devices")
    //         .takes_value(true)
    //     )
    //     .arg(Arg::with_name("script file")
    //         .help("Executes the given script file")
    //         .index(1)
    //         .required(true))
    //     .get_matches();

    let device_list_config_name = "devices.list";

    let xdg_dirs = BaseDirectories::with_prefix("map2")
        .map_err(|_| anyhow!("failed to initialize XDG directory configuration"))?;

    // let script_path = matches.value_of("script file").unwrap().to_string();
    // let script_file = fs::File::open(&script_path)
    //     .map_err(|err| anyhow!("failed to read script file '{}': {}", &script_path, &err))?;


    let device_list_path = PathBuf::from("local/device.list");

    let file =
        fs::File::open(PathBuf::from(&device_list_path)).unwrap();
    // .map_err(|err| anyhow!("failed to open device list '{}': {}", &path.display(), err))
    // });

    let device_list = BufReader::new(file)
        .lines()
        .collect::<std::result::Result<_, _>>()
        .map_err(|err| anyhow!("failed to parse devices file: {}", err))?;

    // let verbosity = matches.occurrences_of("verbosity") as i32;
    //
    let config = Configuration {
        // script_file,
        // verbosity,
        devices: device_list,
    };

    Ok(config)
}