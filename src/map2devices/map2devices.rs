use map2::*;
use std::env;
use std::io::Read;
use std::fs;
use std::path::{Path, PathBuf};
use ncurses::*;
use walkdir::WalkDir;
use std::str::FromStr;
use regex::Regex;
use evdev_rs::{Device, DeviceWrapper};
use std::fs::OpenOptions;
use std::collections::hash_map::Entry;

fn open_file() -> fs::File {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2
    {
        println!("Usage:\n\t{} <rust file>", args[0]);
        println!("Example:\n\t{} examples/ex_3.rs", args[0]);
        panic!("Exiting");
    }

    let reader = fs::File::open(Path::new(&args[1]));
    reader.ok().expect("Unable to open file")
}

fn prompt() {
    addstr("<-Press Any Key->");
    getch();
}

fn get_fd_list() -> Vec<PathBuf> {
    let mut list = vec![];
    // let pattern = Regex::new(&*pattern_str)?;

    for entry in WalkDir::new("/dev/input")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_file())
    {
        // let name: String = String::from(entry.path().to_string_lossy());

        // if !patterns.iter().any(|p| p.is_match(&name)) { continue; }
        // if !pattern.is_match(&name) { continue; }


        // list.push(PathBuf::from_str(&name).unwrap());
        list.push(entry.path().to_path_buf());
    }
    list
}

fn filter_fd_list<'a>(fd_list: &'a Vec<PathBuf>, device_map: &HashMap<PathBuf, Option<DeviceInfo>>, pattern_str: &String) -> Result<Vec<&'a PathBuf>> {
    let mut filtered_list = vec![];
    let pattern = Regex::new(&*pattern_str)?;

    for fd_path in fd_list {
        // match against fd path
        if !pattern.is_match(&*fd_path.to_string_lossy()) {
            // try to match device info fields
            if let Some(Some(info)) = device_map.get(fd_path) {
                let mut matched = false;
                for field in [&info.name, &info.phys, &info.uniq].iter() {
                    if pattern.is_match(field) { matched = true; }
                }
                if !matched { continue; }
            } else { // no device info and fd didn't match, skip
                continue;
            }
        }

        filtered_list.push(fd_path);
    }
    Ok(filtered_list)
}

struct DeviceInfo {
    fd: PathBuf,
    name: String,
    phys: String,
    uniq: String,
}

fn get_props(fd: PathBuf) -> Result<DeviceInfo> {
    let file = OpenOptions::new()
        .read(true)
        .open(&fd)?;

    let device = Device::new_from_file(file)?;

    Ok(DeviceInfo {
        fd,
        name: device.name().unwrap_or("None").to_string(),
        phys: device.phys().unwrap_or("None").to_string(),
        uniq: device.uniq().unwrap_or("None").to_string(),
    })
}

fn main() {
    // let reader = open_file().bytes();

    /* Start ncurses. */
    initscr();
    keypad(stdscr(), true);
    noecho();

    /* Get the screen bounds. */
    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let mut filter = String::new();
    let prompt_height = 1;
    let entry_height = 2;

    let mut device_map = HashMap::new();

    let mut fd_list;
    loop {
        clear();

        fd_list = get_fd_list();
        if let Ok(filtered_fd_list) = filter_fd_list(&fd_list, &device_map, &filter) {
            // let mut visible_len = i32::min(filtered_fd_list.len() as i32, max_y);
            //
            // let mut skip = list.len() as i32 - visible_len + prompt_height;
            // skip = skip + visible_len / entry_height;

            let mut remaining_lines = max_y - prompt_height;

            for (idx, &fd_path) in filtered_fd_list.iter().rev().enumerate() {
                addstr(&*fd_path.to_string_lossy());
                addch('\n' as chtype);

                let device_info = match device_map.entry(fd_path.clone()) {
                    Entry::Occupied(o) => o.into_mut(),
                    Entry::Vacant(v) => v.insert(get_props(fd_path.clone()).ok())
                };

                if let Some(device_info) = device_info {
                    if remaining_lines < 2 { break; }
                    remaining_lines = remaining_lines - 2;

                    addstr(&*format!("  {{name: '{}', phys: '{}', uniq: '{}'}}\n", device_info.name, device_info.phys, device_info.uniq));
                } else {
                    if remaining_lines < 1 { break; }
                    remaining_lines = remaining_lines - 1;

                    // TODO show errors in verbose mode
                }

                // if idx == 2 { attron(A_BOLD() | A_BLINK()); }
                // if idx == 2 { attroff(A_BOLD() | A_BLINK()); }
            }
        } else {
            // failed creating pattern, show nothing
        }


        addch('\n' as chtype);
        addstr(&*format!("search: {}", filter));

        fn process_input(ch: i32, filter: &mut String) {
            match ch {
                // backspace
                127 => { filter.pop(); }
                // ctrl+w
                23 => { filter.clear(); }
                _ => { filter.push(ch as u8 as char); }
            }
        }

        process_input(getch(), &mut filter);
    }

    /* Terminate ncurses. */
    mv(max_y - 1, 0);
    prompt();
    endwin();
}
