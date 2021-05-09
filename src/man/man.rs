use man::prelude::*;

fn main() {
    let page = Manual::new("Key\\ mods")
        .about("A scripting language that allows complex key remapping on Linux.")
        .author(Author::new("shiro").email("shiro@usagi.io"))
        .flag(Flag::new()
            .help("Prints verbose information")
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
            .command("key-mods example.km")
            .output("Runs the specified script.")
        )
        .example(Example::new()
            .text("run a script and capture devices matched by the device list")
            .command("key-mods -d device.list example.km")
            .output("Captures devices that match the selectors in `device.list` and runs the script.")
        )
        .custom(
            Section::new("devices")
                .paragraph(&*vec![
                    "In order to capture device input it is necessary to configure which devices should get captured.",
                    "A list of devices can be specified by providing a device list argument or by defining a default configuration",
                    "in the user's configuration directory ($XDG_CONFIG_HOME/key-mods/device.list).",
                ].join(" ")))
        .custom(
            Section::new("license")
                .paragraph("MIT")
        )
        .render();

    println!("{}", page);
}
