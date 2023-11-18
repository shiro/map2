// use std::collections::hash_map::Entry;
// use std::collections::HashSet;
// use std::fs::OpenOptions;
// use std::path::PathBuf;
//
// use evdev_rs::{Device, DeviceWrapper};
// use ncurses::*;
// use regex::RegexBuilder;
// use tokio::sync::mpsc;
// use walkdir::WalkDir;
//
// use map2::*;
//
// fn get_fd_list() -> Vec<PathBuf> {
//     let mut list = vec![];
//
//     for entry in WalkDir::new("/dev/input")
//         .into_iter()
//         .filter_map(Result::ok)
//         .filter(|e| !e.file_type().is_file())
//     {
//         list.push(entry.path().to_path_buf());
//     }
//     list
// }
//
// fn filter_fd_list<'a>(fd_list: &'a Vec<PathBuf>, device_map: &HashMap<PathBuf, Option<DeviceInfo>>, pattern_str: &str) -> Result<Vec<&'a PathBuf>> {
//     let mut filtered_list = vec![];
//     let pattern = RegexBuilder::new(&*pattern_str)
//         .case_insensitive(true)
//         .build()?;
//
//     for fd_path in fd_list {
//         // match against fd path
//         if !pattern.is_match(&*fd_path.to_string_lossy()) {
//             // try to match device info fields
//             if let Some(Some(info)) = device_map.get(fd_path) {
//                 let mut matched = false;
//                 for field in [&info.name, &info.phys, &info.uniq].iter() {
//                     if pattern.is_match(field) { matched = true; }
//                 }
//                 if !matched { continue; }
//             } else { // no device info and fd didn't match, skip
//                 continue;
//             }
//         }
//
//         filtered_list.push(fd_path);
//     }
//     Ok(filtered_list)
// }
//
// struct DeviceInfo {
//     name: String,
//     phys: String,
//     uniq: String,
// }
//
// #[derive(Clone)]
// enum Mode {
//     Normal,
//     Search,
// }
//
// fn get_props(fd: PathBuf, _reader_tx: mpsc::UnboundedSender<PathBuf>) -> Result<DeviceInfo> {
//     let file = OpenOptions::new()
//         .read(true)
//         .open(&fd)?;
//
//     let device = Device::new_from_file(file)?;
//
//     let device_info = DeviceInfo {
//         name: device.name().unwrap_or("None").to_string(),
//         phys: device.phys().unwrap_or("None").to_string(),
//         uniq: device.uniq().unwrap_or("None").to_string(),
//     };
//     // let start = time::Instant::now();
//
//     // open listen thread
//     // TODO fix this legacy code, not sure what it does
//     // std::thread::spawn(move || {
//     //     read_from_device_input_fd_thread_handler(
//     //         device,
//     //         Arc::new(|_, _| {
//     //             if start.elapsed() < time::Duration::from_millis(100) { return; }
//     //             start = time::Instant::now();
//     //
//     //             let _ = reader_tx.send(fd.clone());
//     //         }),
//     //         ::oneshot::channel().1,
//     //     );
//     // });
//
//     Ok(device_info)
// }
//
// enum ProcessInputRet {
//     None,
//     Quit,
// }
//
// fn process_input(ch: i32, mode: &mut Mode, filter: &mut String, scroll_offset: &mut usize) -> ProcessInputRet {
//     use Mode::*;
//     match mode {
//         Normal => {
//             match ch {
//                 // ctrl + d
//                 4 => { *scroll_offset = *scroll_offset + 5; }
//                 // ctrl + u
//                 21 => {
//                     if *scroll_offset >= 5 {
//                         *scroll_offset = *scroll_offset - 5;
//                     } else {
//                         *scroll_offset = 0
//                     }
//                 }
//                 // j
//                 106 => { *scroll_offset = *scroll_offset + 1; }
//                 // k
//                 107 => { if *scroll_offset != 0 { *scroll_offset = *scroll_offset - 1; } }
//                 // q
//                 113 => { return ProcessInputRet::Quit; }
//                 // slash
//                 47 => { *mode = Search; }
//                 _ => {}
//             }
//         }
//         Search => {
//             match ch {
//                 // enter
//                 10 => { *mode = Normal; }
//                 // esc
//                 27 => {
//                     filter.clear();
//                     *mode = Normal;
//                 }
//                 // backspace
//                 127 => { let _ = filter.pop(); }
//                 // ctrl+w
//                 23 => { filter.clear(); }
//                 _ => { filter.push(ch as u8 as char); }
//             }
//         }
//     }
//     ProcessInputRet::None
// }
//
// #[tokio::main]
// async fn main() {
//     initscr();
//     keypad(stdscr(), true);
//     noecho();
//
//     // loop { println!("{}", getch()); }
//
//     /* Get the screen bounds. */
//     let mut max_x = 0;
//     let mut max_y = 0;
//     getmaxyx(stdscr(), &mut max_y, &mut max_x);
//
//     let mut filter = String::new();
//     let prompt_height = 1;
//
//     let mut device_map = HashMap::new();
//
//     // all device input event updates are received through the channel
//     let (fd_ev_tx, mut fd_ev_rx) = mpsc::unbounded_channel();
//
//     let (ch_tx, mut ch_rx) = mpsc::channel(16);
//     std::thread::spawn(move || {
//         loop {
//             let ch = getch();
//             futures::executor::block_on(ch_tx.send(ch)).unwrap();
//         }
//     });
//
//
//     let (fd_ev_combined_tx, mut fd_ev_combined_rx) = mpsc::channel(8);
//     std::thread::spawn(move || {
//         loop {
//             let mut fd_set = HashSet::new();
//
//             while let Ok(fd_path) = fd_ev_rx.try_recv() {
//                 fd_set.insert(fd_path);
//             }
//
//             let _ = futures::executor::block_on(fd_ev_combined_tx.send(fd_set));
//             std::thread::sleep(Duration::from_millis(500));
//         }
//     });
//
//
//     let mut update = move |mode: &Mode, filter: &str, highlight_set: &HashSet<PathBuf>, scroll_offset: &mut usize| {
//         clear();
//
//         let fd_list = get_fd_list();
//         if let Ok(filtered_fd_list) = filter_fd_list(&fd_list, &device_map, &filter) {
//             let mut remaining_lines = max_y - prompt_height;
//             // make sure we can't overscroll
//             let max_scroll_offset = i32::max(0, filtered_fd_list.len() as i32 - remaining_lines) as usize;
//             *scroll_offset = usize::min(*scroll_offset, max_scroll_offset);
//
//             for &fd_path in filtered_fd_list.iter().skip(*scroll_offset).take(remaining_lines as usize) {
//                 let device_info = match device_map.entry(fd_path.clone()) {
//                     Entry::Occupied(o) => o.into_mut(),
//                     Entry::Vacant(v) => v.insert(get_props(fd_path.clone(), fd_ev_tx.clone()).ok())
//                 };
//
//                 if highlight_set.contains(fd_path) { attron(A_REVERSE()); }
//
//                 if let Some(device_info) = device_info {
//                     if remaining_lines < 2 { break; }
//                     remaining_lines = remaining_lines - 2;
//
//                     addstr(&*fd_path.to_string_lossy());
//                     addch('\n' as chtype);
//                     addstr(&*format!("  {{name: '{}', phys: '{}', uniq: '{}'}}\n", device_info.name, device_info.phys, device_info.uniq));
//                 } else {
//                     if remaining_lines < 1 { break; }
//                     remaining_lines = remaining_lines - 1;
//
//                     addstr(&*fd_path.to_string_lossy());
//                     addch('\n' as chtype);
//                     // TODO show errors in verbose mode
//                 }
//
//                 attroff(A_REVERSE());
//             }
//         } else {
//             addstr("no results, invalid search pattern");
//         }
//
//         match mode {
//             Mode::Normal => {}
//             Mode::Search => {
//                 addch('\n' as chtype);
//                 addstr(&*format!("search: {}", &filter));
//             }
//         }
//     };
//
//     let mut mode = Mode::Normal;
//     let mut highlight_set = HashSet::new();
//     let mut scroll_offset = 0;
//     update(&mode, "", &highlight_set, &mut scroll_offset);
//
//     loop {
//         refresh();
//         tokio::select! {
//             Some(ch) = ch_rx.recv() => {
//                 let ret = process_input(ch, &mut mode, &mut filter, &mut scroll_offset);
//                 match ret{
//                     ProcessInputRet::Quit => {
//                         mv(max_y - 1, 0);
//                         endwin();
//                         std::process::exit(0);
//                     }
//                     _ => {}
//                 }
//             }
//             Some(fd_set) = fd_ev_combined_rx.recv() => {
//                 highlight_set = fd_set;
//             }
//         }
//
//         update(&mode, &filter, &highlight_set, &mut scroll_offset);
//     }
// }
