use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{mpsc, Arc};
use std::{fs, io, thread, time};

use crate::EvdevInputEvent;
use anyhow::{anyhow, Result};
use evdev_rs::{Device, GrabMode, InputEvent, ReadFlag, ReadStatus};
use notify::{DebouncedEvent, Watcher};
use regex::Regex;
use tokio::io::unix::AsyncFd;
use uuid::Uuid;
use walkdir::WalkDir;

fn udev_info(fd_path: &Path) -> Option<udev::Device> {
    let metadata = fs::metadata(fd_path).unwrap_or_else(|_| panic!("Can't open file: {:?}", fd_path));
    let devtype = match std::os::linux::fs::MetadataExt::st_mode(&metadata) & libc::S_IFMT {
        libc::S_IFCHR => udev::DeviceType::Character,
        libc::S_IFBLK => udev::DeviceType::Block,
        _ => return None,
    };

    let ud = match udev::Device::from_devnum(devtype, std::os::linux::fs::MetadataExt::st_rdev(&metadata)) {
        Ok(v) => v,
        Err(_) => return None,
    };
    Some(ud)
}

fn find_fd_with_pattern(fd_path: &PathBuf, udev: &udev::Device, matchers: &Vec<ParsedDeviceMatcher>) -> bool {
    matchers.iter().filter(|matcher| !matcher.is_empty()).cloned().any(|mut matcher| {
        use std::collections::hash_map::Entry::Occupied;

        if let Some(query) = matcher.remove("path") {
            if !query.find(fd_path.to_str().unwrap()).map_or(false, |m| m.len() == fd_path.to_str().unwrap().len()) {
                return false;
            }
        }

        let mut curr_ud = Some(udev.clone());
        while let Some(ud) = curr_ud {
            for prop in ud.properties() {
                let key = prop.name().to_str().unwrap().to_lowercase();
                if let Occupied(entry) = matcher.entry(key.to_string()) {
                    let value = prop.value().to_str().unwrap();
                    let value = &value[1..value.len() - 1];

                    if entry.get().find(&value).map_or(false, |m| m.len() == value.len()) {
                        entry.remove();
                    }
                }
            }
            if matcher.is_empty() {
                return true;
            }
            curr_ud = ud.parent();
        }

        false
    })
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
                    Some(libc::EWOULDBLOCK) => {}
                    _ => {
                        println!("Reader event polling loop error: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum GrabDeviceError {
    #[error("Failed to open device '{0}'")]
    FailedToOpenDevice(String),
    #[error("Failed to grab device '{0}'")]
    FailedToGrabDevice(String),
    #[error("Other")]
    Other,
}

fn grab_device(
    fd_path: &Path,
    ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
) -> Result<oneshot::Sender<()>, GrabDeviceError> {
    use nix::fcntl::{FcntlArg, OFlag};

    let fd_file = fs::OpenOptions::new()
        .read(true)
        .open(&fd_path)
        .map_err(|err| GrabDeviceError::FailedToOpenDevice(fd_path.to_string_lossy().to_string()))?;

    // let fd_file = pyo3_asyncio::tokio::get_runtime().block_on(async {
    //         AsyncFd::new(fd_file).unwrap()
    //     });

    nix::fcntl::fcntl(fd_file.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK))
        .map_err(|err| GrabDeviceError::Other)?;

    let mut device = Device::new_from_file(fd_file)
        .map_err(|err| GrabDeviceError::FailedToOpenDevice(fd_path.to_string_lossy().to_string()))?;
    device
        .grab(GrabMode::Grab)
        .map_err(|err| GrabDeviceError::FailedToGrabDevice(fd_path.to_string_lossy().to_string()))?;

    // spawn tasks for reading devices
    let (abort_tx, abort_rx) = oneshot::channel();
    pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
        read_from_device_input_fd_thread_handler(device, ev_handler, abort_rx).await;
    });

    Ok(abort_tx)
}

pub type DeviceMatcher = HashMap<String, String>;
type ParsedDeviceMatcher = HashMap<String, Regex>;
// pub struct DeviceFilter {
//     path: PathBuf,
//     name: String,
//     phys: String,
//     uniq: String,
//     attributes: HashMap<String, String>,
// }

pub fn grab_udev_inputs(
    // fd_patterns: &[impl AsRef<str>],
    matchers: Vec<DeviceMatcher>,
    ev_handler: Arc<impl Fn(&str, EvdevInputEvent) + Send + Sync + 'static>,
    exit_rx: oneshot::Receiver<()>,
) -> Result<thread::JoinHandle<Result<()>>> {
    let parsed_matchers = matchers
        .into_iter()
        // .map(|x| Regex::new(x.as_ref()))
        .map(|x| {
            Ok(x.into_iter()
                .map(|(k, v)| {
                    let regex = Regex::new(&v).unwrap();
                    // .map_err(|err| Err(anyhow!(""))).unwrap();
                    Ok((k, regex))
                })
                .collect::<Result<HashMap<String, Regex>>>()?)
        })
        // .map(|v| -> Result<ParsedDeviceMatcher> {
        //     let v = v?;
        //     if (!v.contains_key("path")) {
        //         return Err(anyhow!("no path specified"));
        //     }
        //     Ok(v)
        // })
        .collect::<Result<Vec<ParsedDeviceMatcher>>>()
        // .map_err(|err| Err(anyhow!("failed to parse regex")))?;
        .unwrap();

    // devices are monitored and hooked up when added/removed, so we need another thread
    let join_handle = thread::spawn(move || {
        let (fs_ev_tx, fs_ev_rx) = mpsc::channel();

        let mut watcher: notify::RecommendedWatcher = notify::Watcher::new(fs_ev_tx, time::Duration::from_secs(2))?;
        watcher.watch("/dev/input", notify::RecursiveMode::Recursive)?;

        let mut device_map = HashMap::new();

        // grab all devices
        for entry in WalkDir::new("/dev/input").into_iter().filter_map(Result::ok).filter(|e| !e.file_type().is_file())
        {
            let fd_path = entry.path().to_owned();
            let udev = if let Some(v) = udev_info(&fd_path) { v } else { continue };

            if device_map.contains_key(udev.syspath()) {
                continue;
            }
            if !find_fd_with_pattern(&fd_path, &udev, &parsed_matchers) {
                continue;
            }
            let res = grab_device(&fd_path, ev_handler.clone());
            let abort_tx = match res {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };

            println!("add {:?}", fd_path.clone());
            device_map.insert(udev.syspath().to_owned(), abort_tx);
        }

        // continuously check if devices are added/removed and handle it
        loop {
            if let Ok(()) = exit_rx.try_recv() {
                return Ok(());
            }

            while let Ok(event) = fs_ev_rx.try_recv() {
                match event {
                    DebouncedEvent::Create(fd_path) => {
                        let udev = if let Some(v) = udev_info(&fd_path) { v } else { continue };

                        if device_map.contains_key(udev.syspath()) {
                            continue;
                        }
                        if !find_fd_with_pattern(&fd_path, &udev, &parsed_matchers) {
                            continue;
                        }

                        let abort_tx = grab_device(&fd_path, ev_handler.clone())?;
                        println!("add {:?}", fd_path.clone());
                        device_map.insert(udev.syspath().to_owned(), abort_tx);
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
