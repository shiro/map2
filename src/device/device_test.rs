use std::{io, time};
use std::collections::HashMap;
use std::fs::File;

use anyhow::{anyhow, bail, Result};
use evdev_rs::*;
use evdev_rs::enums::*;
use futures::{SinkExt, TryFutureExt};
use itertools::enumerate;
use notify::{DebouncedEvent, Watcher};
use regex::Regex;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task;
use walkdir::{DirEntry, WalkDir};

use crate::device::{device_util, virt_device};

fn get_fd_list(patterns: &Vec<Regex>) -> Vec<String> {
    let mut list = vec![];
    for entry in WalkDir::new("/dev/input")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_file())
    {
        let name: String = String::from(entry.path().to_string_lossy());

        if !patterns.iter().any(|p| p.is_match(&name)) { continue; }
        list.push(name);
    }
    list
}


async fn read_from_fd_runner(mut device: Device, reader_rx: mpsc::Sender<InputEvent>,
                             mut return_broadcast_rx: broadcast::Receiver<()>,
) {
    let mut a: io::Result<(ReadStatus, InputEvent)>;
    loop {
        // println!("spin");
        if return_broadcast_rx.try_recv().is_ok() {
            // device.grab(GrabMode::Ungrab).unwrap();
            // drop(device);
            return;
        }

        a = device.next_event(ReadFlag::NORMAL);
        // println!("wow");
        if a.is_ok() {
            // let mut result = a.ok().unwrap();
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
                ReadStatus::Success => {
                    reader_rx.send(result.1).await;
                }
            }
        } else {
            let err = a.err().unwrap();
            match err.raw_os_error() {
                Some(libc::EAGAIN) => {
                    // println!("yielded");
                    task::yield_now().await;
                    continue;
                }
                _ => {
                    println!("{}", err);
                    break;
                }
            }
        }
    }
}


async fn runner_it(fd_pattens: &Vec<Regex>, mut reader_rx: mpsc::Receiver<InputEvent>, mut writer: &mpsc::Sender<InputEvent>,
                   mut return_rx: oneshot::Receiver<oneshot::Sender<mpsc::Receiver<InputEvent>>>,
                   mut return_broadcast_rx: broadcast::Sender<()>,
)
                   -> Result<()> {
    // let mut first_iteration_box = Some((reader_rx, return_rx));
    // if let Some((mut reader_rx, mut return_tx)) = first_iteration_box {
    let file = tokio::fs::File::open("/tmp/dummy").await.unwrap();

    let file_nb = tokio_file_unix::File::new_nb(file).unwrap();

    let mut new_device = UninitDevice::new().unwrap().unstable_force_init(file_nb);
    // device_util::clone_device(&device, &mut new_device)?;
    virt_device::setup_virt_device(&mut new_device);

    let input_device = UInputDevice::create_from_device(&new_device).unwrap();


    // let input_device = UInputDevice::create_from_device(&device)?;
    task::spawn(async move {
        loop {
            tokio::select! {
                return_tx = &mut return_rx => {
                    return_tx.unwrap().send(reader_rx);
                    return Ok(());
                }
                msg = reader_rx.recv() => {
                    let ev: InputEvent = match msg {
                        Some(v) => v,
                        None => return Err(anyhow!("message channel closed unexpectedly")),
                    };
                    input_device.write_event(&ev);
                }
            }
        }
        Ok(())
    });


    let input_fd_path_list = get_fd_list(&fd_pattens);
    if input_fd_path_list.len() < 1 {
        return Err(anyhow!("no devices found"));
    }

    // println!("{:?}", input_fd_path_list);


    let mut devices = vec![];
    for fd_path in input_fd_path_list {
        // if fd_map.get(&fd_path).is_some() { continue; }
        // fd_map.insert(fd_path.clone(), true);

        // grab fd_path as fd
        let fd_file = tokio::fs::File::open(fd_path).await?;
        let fd_file_nb = tokio_file_unix::File::new_nb(fd_file).unwrap();
        let mut device = Device::new_from_file(fd_file_nb)?;
        device.grab(GrabMode::Grab).unwrap();
        devices.push(device);
    }

    for device in devices {
        // spawn tasks for reading devices
        let writer = writer.clone();
        let return_broadcast_rx = return_broadcast_rx.subscribe();
        task::spawn(async move {
            read_from_fd_runner(device, writer, return_broadcast_rx).await;
        });
    }

    Ok(())
}

async fn runner(fd_pattens: Vec<Regex>, reader_init: oneshot::Sender<mpsc::Sender<InputEvent>>, mut writer: mpsc::Sender<InputEvent>)
                -> Result<()> {
    task::spawn(async move {
        let (fs_event_tx, mut fs_event_rx) = mpsc::channel(128);
        task::spawn_blocking(move || {
            let (watch_tx, watch_rx) = std::sync::mpsc::channel();
            let mut watcher: notify::RecommendedWatcher = notify::Watcher::new(watch_tx, time::Duration::from_secs(2))?;
            watcher.watch("/dev/input", notify::RecursiveMode::Recursive)?;

            loop {
                match watch_rx.recv() {
                    Ok(event) => {
                        match event {
                            DebouncedEvent::Create(_) => {}
                            DebouncedEvent::Remove(_) => {}
                            _ => { continue; }
                        }

                        println!("ev: {:?}", event);
                        futures::executor::block_on(
                            fs_event_tx.send(())
                            // .map_err(|_| Err(anyhow!("fs message channel sync error")))
                        );
                    }
                    Err(e) => return Err(anyhow!("watch error: {:?}", e)),
                }
            }
            Ok(())
        });


        let (reader_tx, mut reader_rx) = mpsc::channel(128);
        // send the reader to the client
        reader_init.send(reader_tx.clone());

        // put it in an option to avoid moved var errors
        let mut reader_rx_box = Some(reader_rx);

        // no return channel for the first iteration
        let mut return_tx_box: Option<oneshot::Sender<oneshot::Sender<mpsc::Receiver<InputEvent>>>> = None;

        let (mut broadcast, _) = broadcast::channel(64);

        loop {
            match return_tx_box {
                Some(mut return_tx) => {
                    broadcast.send(())?;

                    let (tx, rx) = oneshot::channel();
                    return_tx.send(tx).unwrap();
                    reader_rx_box = Some(rx.await.unwrap());
                }
                None => {} // first iteration, nothing to retreive
            }
            let (return_tx, return_rx) = oneshot::channel();
            return_tx_box = Some(return_tx);

            println!("new it");
            runner_it(&fd_pattens, reader_rx_box.unwrap(), &writer, return_rx, broadcast.clone()).await;
            reader_rx_box = None;

            fs_event_rx.recv().await;
        }
        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}


pub(crate) async fn bind_udev_inputs(fd_patterns: Vec<&str>, reader_init_tx: oneshot::Sender<mpsc::Sender<InputEvent>>, mut writer_tx: mpsc::Sender<InputEvent>) -> Result<()> {
    if fd_patterns.len() < 1 { bail!(anyhow!("need at least 1 pattern")); }

    let fd_patterns_regex = fd_patterns.into_iter().map(|v| Regex::new(v).unwrap()).collect();

    task::spawn(async move {
        runner(fd_patterns_regex, reader_init_tx, writer_tx).await;
    });

    Ok(())
}
