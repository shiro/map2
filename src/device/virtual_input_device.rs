use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{mpsc, Arc};
use std::{fs, io, thread, time};

use crate::EvdevInputEvent;
use anyhow::{anyhow, Result};
use evdev_rs::*;
use notify::{DebouncedEvent, Watcher};
use regex::Regex;
use tokio::io::unix::AsyncFd;
use uuid::Uuid;
use walkdir::WalkDir;

fn find_fd_with_pattern(patterns: &Vec<Regex>) -> Vec<PathBuf> {
    let mut list = vec![];
    for entry in WalkDir::new("/dev/input").into_iter().filter_map(Result::ok).filter(|e| !e.file_type().is_file()) {
        let name: String = String::from(entry.path().to_string_lossy());

        // pattens need to match the whole name
        if !patterns.iter().any(|p| p.find(&name).map_or(false, |m| m.len() == name.len())) {
            continue;
        }
        list.push(PathBuf::from_str(&name).unwrap());
    }
    list
}

pub async fn read_from_device_input_fd_thread_handler(
    device: Device,
    ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
    abort_rx: oneshot::Receiver<()>,
) {
    let mut read_buf: io::Result<(ReadStatus, InputEvent)>;
    let id = Uuid::new_v4().to_string();

    let file = device.file().as_ref().unwrap().as_raw_fd();
    let async_fd = AsyncFd::new(file).unwrap();

    loop {
        if abort_rx.try_recv().is_ok() {
            return;
        }

        let mut guard = async_fd.readable().await.unwrap();
        guard.clear_ready();

        loop {
            read_buf = device.next_event(ReadFlag::NORMAL);
            if read_buf.is_ok() {
                let mut result = read_buf.ok().unwrap();
                match result.0 {
                    ReadStatus::Sync => {
                        // dropped, need to sync
                        while result.0 == ReadStatus::Sync {
                            read_buf = device.next_event(ReadFlag::SYNC);
                            if read_buf.is_ok() {
                                result = read_buf.ok().unwrap();
                            } else {
                                // something failed, abort sync and carry on
                                break;
                            }
                        }
                    }
                    ReadStatus::Success => {
                        ev_handler(&id, result.1);
                    }
                }
            } else {
                let err = read_buf.err().unwrap();
                match err.raw_os_error() {
                    // Some(libc::ENODEV) => { return; }
                    Some(libc::EWOULDBLOCK) => {
                        // println!("would block!");
                        // thread::sleep(time::Duration::from_millis(10));
                        // thread::yield_now();
                        break;
                    }
                    _ => {
                        println!("Reader event polling loop error: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

pub trait Sendable<T> {
    fn send(&self, t: T);
}

fn grab_device(
    fd_path: &Path,
    ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
) -> Result<oneshot::Sender<()>> {
    use nix::fcntl::{FcntlArg, OFlag};

    let fd_file = fs::OpenOptions::new()
        .read(true)
        .open(&fd_path)
        .map_err(|err| anyhow!("failed to open fd '{}': {err}", fd_path.to_str().unwrap_or("...")))?;

    // let fd_file = pyo3_asyncio::tokio::get_runtime().block_on(async {
    //         AsyncFd::new(fd_file).unwrap()
    //     });

    nix::fcntl::fcntl(fd_file.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?;

    let mut device = Device::new_from_file(fd_file)
        .map_err(|err| anyhow!("failed to open fd '{}': {err}", fd_path.to_str().unwrap_or("...")))?;
    device
        .grab(GrabMode::Grab)
        .map_err(|err| anyhow!("failed to grab device '{}': {}", fd_path.to_string_lossy(), err))?;

    // spawn tasks for reading devices
    let (abort_tx, abort_rx) = oneshot::channel();
    pyo3_asyncio::tokio::get_runtime().spawn(async move {
        read_from_device_input_fd_thread_handler(device, ev_handler, abort_rx).await;
    });

    Ok(abort_tx)
}

pub fn grab_udev_inputs(
    fd_patterns: &[impl AsRef<str>],
    ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
    exit_rx: oneshot::Receiver<()>,
) -> Result<thread::JoinHandle<Result<()>>> {
    let device_fd_path_pattens = fd_patterns
        .into_iter()
        .map(|x| Regex::new(x.as_ref()))
        .collect::<std::result::Result<_, _>>()
        .map_err(|err| anyhow!("failed to parse regex: {}", err))?;

    // devices are monitored and hooked up when added/removed, so we need another thread
    let join_handle = thread::spawn(move || {
        let (fs_ev_tx, fs_ev_rx) = mpsc::channel();

        let mut watcher: notify::RecommendedWatcher = notify::Watcher::new(fs_ev_tx, time::Duration::from_secs(2))?;
        watcher.watch("/dev/input", notify::RecursiveMode::Recursive)?;

        let mut device_map = HashMap::new();

        // grab all devices
        for device_fd_path in find_fd_with_pattern(&device_fd_path_pattens) {
            let res = grab_device(&device_fd_path, ev_handler.clone());
            let abort_tx = match res {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };

            device_map.insert(device_fd_path, abort_tx);
        }

        // continuously check if devices are added/removed and handle it
        loop {
            if let Ok(()) = exit_rx.try_recv() {
                return Ok(());
            }

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
                    _ => {
                        continue;
                    }
                };
            }

            thread::sleep(time::Duration::from_millis(100));
            thread::yield_now();
        }
    });

    Ok(join_handle)
}
