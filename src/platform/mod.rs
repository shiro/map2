use std::process::Command;

pub enum Platform {
    Hyprland,
    X11,
    Unknown,
}

pub fn get_platform() -> Platform {
    if platform_is_hyprland() {
        return Platform::Hyprland;
    }
    if platform_is_x11() {
        return Platform::X11;
    }
    Platform::Unknown
}

fn platform_is_hyprland() -> bool {
    Command::new("printenv")
        .arg("HYPRLAND_INSTANCE_SIGNATURE")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn platform_is_x11() -> bool {
    Command::new("printenv")
        .arg("XDG_SESSION_TYPE")
        .output()
        .map(|info| {
            info.status.success()
                && String::from_utf8_lossy(&info.stdout).replace("\n", "") == "x11"
        })
        .unwrap_or(false)
}

// fn platform_is_sway() -> bool {
//     Command::new("printenv")
//         .arg("SWAYSOCK")
//         .status()
//         .map(|status| status.success())
//         .unwrap_or(false)
// }

// for kde/kwin (wayland)
// https://unix.stackexchange.com/questions/706477/is-there-a-way-to-get-list-of-windows-on-kde-wayland

// for gnome
// https://github.com/ActivityWatch/aw-watcher-window/pull/46/files

