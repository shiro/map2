use man::prelude::*;

fn main() {
    let page = Manual::new("Map2")
        .about("A scripting language that allows complex key remapping on Linux.")
        .author(Author::new("shiro").email("shiro@usagi.io"))
        .flag(Flag::new()
            .help("Sets the verbosity level")
            .short("-v")
            .long("--verbose")
        )
        .flag(Flag::new()
            .help("Selects the input devices")
            .short("-d")
            .long("--devices")
        )
        .example(Example::new()
            .text("run a script")
            .command("map2 example.m2")
            .output("Runs the specified script.")
        )
        .example(Example::new()
            .text("run a script and capture devices matched by the device list")
            .command("map2 -d device.list example.m2")
            .output("Captures devices that match the selectors in `device.list` and runs the script.")
        )
        .example(Example::new()
            .text("run a script with maximum debug output")
            .command("map2 -vvv example.m2")
            .output("Runs the script example.m2 and outputs all debug information.")
        )
        .custom(
            Section::new("devices")
                .paragraph(&*vec![
                    "In order to capture device input it is necessary to configure which devices should get captured.",
                    "A list of devices can be specified by providing a device list argument or by defining a default configuration",
                    "in the user's configuration directory ($XDG_CONFIG_HOME/map2/device.list).",
                ].join(" ")))
        .custom(
            Section::new("license")
                .paragraph("MIT")
        )
        .render();

    println!("{}", page);
}
