use std::{fs, io, thread, time};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, mpsc};

use anyhow::{anyhow, Result};
use evdev_rs::*;
use notify::{DebouncedEvent, Watcher};
use regex::Regex;
use uuid::Uuid;
use walkdir::WalkDir;
use crate::EvdevInputEvent;

fn get_fd_list(patterns: &Vec<Regex>) -> Vec<PathBuf> {
    let mut list = vec![];
    for entry in WalkDir::new("/dev/input")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_file())
    {
        let name: String = String::from(entry.path().to_string_lossy());

        if !patterns.iter().any(|p| p.is_match(&name)) { continue; }
        list.push(PathBuf::from_str(&name).unwrap());
    }
    list
}


pub fn read_from_device_input_fd_thread_handler(
    device: Device,
    mut ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
    abort_rx: oneshot::Receiver<()>,
) {
    let mut a: io::Result<(ReadStatus, InputEvent)>;
    let id = Uuid::new_v4().to_string();
    loop {
        if abort_rx.try_recv().is_ok() { return; }

        a = device.next_event(ReadFlag::NORMAL);
        if a.is_ok() {
            let mut result = a.ok().unwrap();
            match result.0 {
                ReadStatus::Sync => { // dropped, need to sync
                    while result.0 == ReadStatus::Sync {
                        a = device.next_event(ReadFlag::SYNC);
                        if a.is_ok() {
                            result = a.ok().unwrap();
                        } else { // something failed, abort sync and carry on
                            break;
                        }
                    }
                }
                ReadStatus::Success => { ev_handler(&id, result.1); }
            }
        } else {
            let err = a.err().unwrap();
            match err.raw_os_error() {
                Some(libc::ENODEV) => { return; }
                Some(libc::EWOULDBLOCK) => {
                    thread::sleep(time::Duration::from_millis(10));
                    thread::yield_now();
                    continue;
                }
                _ => {
                    println!("{:?}", err);
                    println!("reader loop err: {}", err);
                    return;
                }
            }
        }
    }
}

pub trait Sendable<T> {
    fn send(&self, t: T);
}


fn grab_device
(
    fd_path: &Path,
    ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
)
    -> Result<oneshot::Sender<()>> {
    let fd_file = fs::OpenOptions::new()
        .read(true)
        .open(&fd_path)
        .expect(&*format!("failed to open fd '{}'", fd_path.to_str().unwrap_or("...")));

    let fd_file_nb = tokio_file_unix::File::new_nb(fd_file).unwrap();
    let mut device = Device::new_from_file(fd_file_nb).expect(&*format!("failed to open fd '{}'", fd_path.to_str().unwrap_or("...")));
    device.grab(GrabMode::Grab)
        .map_err(|err| anyhow!("failed to grab device '{}': {}", fd_path.to_string_lossy(), err))?;

    // spawn tasks for reading devices
    let (abort_tx, abort_rx) = oneshot::channel();
    thread::spawn(move || {
        read_from_device_input_fd_thread_handler(
            device,
            // |ev| { let _ = ev_tx.send(ev); },
            ev_handler,
            abort_rx,
        );
    });

    Ok(abort_tx)
}

pub fn grab_udev_inputs
(
    fd_patterns: &[impl AsRef<str>],
    ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
    exit_rx: oneshot::Receiver<()>,
) -> Result<thread::JoinHandle<Result<()>>> {
    let device_fd_path_pattens = fd_patterns.into_iter()
        .map(|v| Regex::new(v.as_ref()))
        .collect::<std::result::Result<_, _>>()
        .map_err(|err| anyhow!("failed to parse regex: {}", err))?;

    // devices are monitored and hooked up when added/removed, so we need another thread
    let join_handle = thread::spawn(move || {
        let (fs_ev_tx, fs_ev_rx) = mpsc::channel();

        let mut watcher: notify::RecommendedWatcher = notify::Watcher::new(fs_ev_tx, time::Duration::from_secs(2))?;
        watcher.watch("/dev/input", notify::RecursiveMode::Recursive)?;

        let mut device_map = HashMap::new();

        // grab all devices
        for device_fd_path in get_fd_list(&device_fd_path_pattens) {
            let res = grab_device(&device_fd_path, ev_handler.clone());
            let abort_tx = match res {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}", err);
                    continue;
                }
            };

            device_map.insert(device_fd_path, abort_tx);
        }

        // continuously check if devices are added/removed and handle it
        loop {
            if let Ok(()) = exit_rx.try_recv() { return Ok(()); }

            while let Ok(event) = fs_ev_rx.try_recv() {
                match event {
                    DebouncedEvent::Create(path) => {
                        // if the device doesn't match any pattern, skip it
                        if !device_fd_path_pattens.iter().any(|regex| regex.is_match(path.to_str().unwrap())) {
                            continue;
                        }

                        let abort_tx = grab_device(&path, ev_handler.clone())?;
                        device_map.insert(path, abort_tx);
                    }
                    DebouncedEvent::Remove(path) => {
                        if let Some(abort_tx) = device_map.remove(&path) {
                            // this might return an error if the device read thread crashed for any reason, ignore it since it was logged already
                            let _ = abort_tx.send(());
                        }
                    }
                    _ => { continue; }
                };
            }

            thread::sleep(time::Duration::from_millis(100));
            thread::yield_now();
        }
    });

    Ok(join_handle)
}

