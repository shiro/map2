use crate::python::*;
use crate::python_util::PyRegex;
use crate::*;
use std::collections::{BTreeMap, HashMap};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::{fs, io, thread, time};

use crate::EvdevInputEvent;
use anyhow::{Result, anyhow};
use evdev_rs::{Device, GrabMode, InputEvent, ReadFlag, ReadStatus};
use notify::Watcher;
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

fn get_udev_properties(device: &udev::Device) -> BTreeMap<String, String> {
    let mut properties = BTreeMap::new();
    let mut current_device = Some(device.clone());

    while let Some(device) = current_device {
        for prop in device.properties() {
            let name = prop.name().to_string_lossy().to_string();
            let value = prop.value().to_string_lossy().to_string();
            properties.entry(name).or_insert(value);
        }
        current_device = device.parent();
    }

    properties
}

pub async fn read_from_device_input_fd_thread_handler(
    device: Device,
    ev_handler: Arc<impl Fn(EvdevInputEvent) + Send + Sync + 'static>,
    // abort_rx: oneshot::Receiver<()>,
) {
    let mut read_buf: io::Result<(ReadStatus, InputEvent)>;

    let file = device.file().as_ref().unwrap().as_raw_fd();
    let async_fd = AsyncFd::new(file).unwrap();

    loop {
        // if abort_rx.try_recv().is_ok() {
        //     return;
        // }

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
                        ev_handler(result.1);
                    }
                }
            } else {
                let err = read_buf.err().unwrap();
                match err.raw_os_error() {
                    Some(libc::ENODEV) => return,
                    Some(libc::EWOULDBLOCK) => {
                        // println!("would block!");
                        // thread::sleep(time::Duration::from_millis(10));
                        // thread::yield_now();
                        break;
                    }
                    // Some(libc::EWOULDBLOCK) => {}
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
pub enum GrabDeviceError {
    #[error("Failed to open device '{0}'")]
    FailedToOpenDevice(String),
    #[error("Failed to grab device '{0}'")]
    FailedToGrabDevice(String),
    #[error("Other")]
    Other,
}

pub fn grab_device(
    fd_path: &Path,
    ev_handler: Arc<impl Fn(EvdevInputEvent) + Send + Sync + 'static>,
) -> Result<tokio::task::AbortHandle, GrabDeviceError> {
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
    let handle = pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
        read_from_device_input_fd_thread_handler(device, ev_handler).await;
    });

    Ok(handle.abort_handle())
}

#[pyclass]
pub struct DeviceMatcher {
    pub path: Option<MatcherValue>,
    pub properties: Option<HashMap<String, MatcherValue>>,
}

pub enum MatcherValue {
    Str(String),
    Regex(PyRegex),
}

impl MatcherValue {
    pub fn is_match(&self, input: &str) -> bool {
        match self {
            MatcherValue::Str(s) => s == input,
            MatcherValue::Regex(r) => Python::with_gil(|py| r.is_match(py, input)),
        }
    }
}

impl<'source> FromPyObject<'source> for MatcherValue {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(regex) = ob.extract::<PyRegex>() {
            return Ok(MatcherValue::Regex(regex));
        }
        if let Ok(string) = ob.extract::<String>() {
            return Ok(MatcherValue::Str(string));
        }
        Err(PyTypeError::new_err("Expected type 'str' or 're.Pattern'"))
    }
}

impl<'source> FromPyObject<'source> for DeviceMatcher {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;

        let path = dict.get_item("path")?.map(|p| p.extract()).transpose()?;
        let properties = dict.get_item("properties")?.map(|p| p.extract()).transpose()?;

        Ok(DeviceMatcher { path, properties })
    }
}

type ParsedDeviceMatcher = HashMap<String, Regex>;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct NativeDeviceInfo {
    pub fd_path: PathBuf,
    pub sys_path: PathBuf,
    pub properties: BTreeMap<String, String>,
}

pub enum NativeDeviceEvent {
    AddDevice(NativeDeviceInfo),
    RemoveDevice(NativeDeviceInfo),
}

pub fn watch_udev_inputs(
    matchers: Vec<DeviceMatcher>,
    ev_handler: Arc<impl Fn(NativeDeviceEvent) + Send + Sync + 'static>,
    mut exit_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<()> {
    let handle_device_event = move |fd_path: PathBuf| -> Option<NativeDeviceInfo> {
        if fd_path.is_dir() {
            return None;
        }

        let udev = if let Some(v) = udev_info(&fd_path) { v } else { return None };
        let properties = get_udev_properties(&udev);

        let all_match = matchers.iter().all(|matcher| {
            matcher.path.as_ref().map_or(true, |pattern| pattern.is_match(&fd_path.to_string_lossy().as_ref()))
                && matcher.properties.as_ref().map_or(true, |props| {
                    props
                        .iter()
                        .all(|(key, pattern)| properties.get(key).map_or(false, |value| pattern.is_match(value)))
                })
        });

        if !all_match {
            return None;
        }

        Some(NativeDeviceInfo { fd_path, sys_path: udev.syspath().to_owned(), properties: get_udev_properties(&udev) })
    };

    // check all devices
    let mut device_map: HashMap<PathBuf, NativeDeviceInfo> = HashMap::new();

    for entry in WalkDir::new("/dev/input").into_iter().filter_map(Result::ok).filter(|e| !e.file_type().is_file()) {
        let fd_path = entry.path().to_owned();
        if let Some(device_info) = handle_device_event(fd_path) {
            if !device_map.contains_key(&device_info.sys_path) {
                device_map.insert(device_info.sys_path.clone(), device_info.clone());
                ev_handler(NativeDeviceEvent::AddDevice(device_info));
            }
        }
    }

    // devices are monitored and hooked up when added/removed, so we need another thread
    pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
        let (fs_ev_tx, mut fs_ev_rx) = tokio::sync::mpsc::channel(32);

        let mut watcher: notify::RecommendedWatcher = notify::recommended_watcher(move |res| {
            futures::executor::block_on(async {
                fs_ev_tx.send(res).await.unwrap();
            })
        })?;
        watcher.watch(Path::new("/dev/input"), notify::RecursiveMode::Recursive)?;

        // continuously check if devices are added/removed and handle it
        tokio::select!(
            Ok(()) = exit_rx => {
                return Ok::<_, anyhow::Error>(());
            },

            (_) = async {
                loop {
                    if let Some(Ok(event)) = fs_ev_rx.recv().await {
                        match event.kind {
                            notify::EventKind::Create(_) => {
                                let fd_path = event.paths.into_iter().next().unwrap();
                                if let Some(device_info) = handle_device_event(fd_path) {
                                    if !device_map.contains_key(&device_info.sys_path) {
                                        device_map.insert(device_info.sys_path.clone(), device_info.clone());
                                        ev_handler(NativeDeviceEvent::AddDevice(device_info));
                                    }
                                }
                            }
                            notify::EventKind::Remove(_) => {
                                let fd_path = event.paths.into_iter().next().unwrap();
                                if let Some(device_info) = handle_device_event(fd_path) {
                                    device_map.remove(&device_info.sys_path);
                                    ev_handler(NativeDeviceEvent::RemoveDevice(device_info));
                                }
                            }
                            _ => { continue; }
                        };
                    }
                }
                // anyhow::Ok(())
                Ok::<_, anyhow::Error>(())
            } => {},
        );
        Ok(())
    });

    Ok(())
}
