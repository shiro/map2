use std::{io, time};
use std::collections::HashMap;
use std::fs::File;

use anyhow::{anyhow, bail, Result};
use evdev_rs::*;
use evdev_rs::enums::*;
use regex::Regex;
use tokio::sync::{mpsc, oneshot, broadcast};
use tokio::task;
use walkdir::{DirEntry, WalkDir};
use crate::device::device_util;
use notify::{Watcher, DebouncedEvent};
use futures::{SinkExt, TryFutureExt};
use itertools::enumerate;

fn usage() {
    println!("Usage: evtest /path/to/device");
}

fn print_abs_bits(dev: &Device, axis: &EV_ABS) {
    let code = EventCode::EV_ABS(axis.clone());

    if !dev.has(&code) {
        return;
    }

    let abs = dev.abs_info(&code).unwrap();

    println!("\tValue\t{}", abs.value);
    println!("\tMin\t{}", abs.minimum);
    println!("\tMax\t{}", abs.maximum);
    if abs.fuzz != 0 {
        println!("\tFuzz\t{}", abs.fuzz);
    }
    if abs.flat != 0 {
        println!("\tFlat\t{}", abs.flat);
    }
    if abs.resolution != 0 {
        println!("\tResolution\t{}", abs.resolution);
    }
}

fn print_code_bits(dev: &Device, ev_code: &EventCode, max: &EventCode) {
    for code in ev_code.iter() {
        if code == *max {
            break;
        }
        if !dev.has(&code) {
            continue;
        }

        println!("    Event code: {}", code);
        match code {
            EventCode::EV_ABS(k) => print_abs_bits(dev, &k),
            _ => (),
        }
    }
}

fn print_bits(dev: &Device) {
    println!("Supported events:");

    for ev_type in EventType::EV_SYN.iter() {
        if dev.has(&ev_type) {
            println!("  Event type: {} ", ev_type);
        }

        match ev_type {
            EventType::EV_KEY => print_code_bits(
                dev,
                &EventCode::EV_KEY(EV_KEY::KEY_RESERVED),
                &EventCode::EV_KEY(EV_KEY::KEY_MAX),
            ),
            EventType::EV_REL => print_code_bits(
                dev,
                &EventCode::EV_REL(EV_REL::REL_X),
                &EventCode::EV_REL(EV_REL::REL_MAX),
            ),
            EventType::EV_ABS => print_code_bits(
                dev,
                &EventCode::EV_ABS(EV_ABS::ABS_X),
                &EventCode::EV_ABS(EV_ABS::ABS_MAX),
            ),
            EventType::EV_LED => print_code_bits(
                dev,
                &EventCode::EV_LED(EV_LED::LED_NUML),
                &EventCode::EV_LED(EV_LED::LED_MAX),
            ),
            _ => (),
        }
    }
}

fn print_props(dev: &Device) {
    println!("Properties:");

    for input_prop in InputProp::INPUT_PROP_POINTER.iter() {
        if dev.has(&input_prop) {
            println!("  Property type: {}", input_prop);
        }
    }
}

pub fn print_event_debug(ev: &InputEvent) {
    match ev.event_code {
        EventCode::EV_SYN(_) => println!(
            "Event: time {}.{}, ++++++++++++++++++++ {} +++++++++++++++",
            ev.time.tv_sec,
            ev.time.tv_usec,
            ev.event_type().unwrap()
        ),
        _ => println!(
            "Event: time {}.{}, type {} , code {} , value {}",
            ev.time.tv_sec,
            ev.time.tv_usec,
            ev.event_type()
                .map(|ev_type| format!("{}", ev_type))
                .unwrap_or("None".to_owned()),
            ev.event_code,
            ev.value
        ),
    }
}


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
        println!("spin");
        if return_broadcast_rx.try_recv().is_ok() {
            println!("exiting");
            device.grab(GrabMode::Ungrab).unwrap();
            drop(device);
            return;
        }

        a = device.next_event(ReadFlag::NORMAL);
        println!("wow");
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
                    println!("yielded");
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
    let mut first_iteration_box = Some((reader_rx, return_rx));

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
        let fd_file = File::open(fd_path)?;
        let mut device = Device::new_from_file(fd_file)?;
        device.grab(GrabMode::Grab).unwrap();
        devices.push(device);
    }

    {
        let mut it = devices.iter_mut();
        let main_dev = it.next().ok_or(anyhow!("failed to open main device"))?;

        for device in it {
            device_util::clone_device(device, main_dev)?;
        }
    }

    for device in devices {
        // spawn virtual device and write to it
        if let Some((mut reader_rx, mut return_tx)) = first_iteration_box {
            let input_device = UInputDevice::create_from_device(&device)?;

            task::spawn(async move {
                loop {
                    if let Ok(return_tx) = return_tx.try_recv() {
                        return_tx.send(reader_rx);
                        return Ok(());
                    } else {
                        let msg: InputEvent = match reader_rx.recv().await {
                            Some(v) => v,
                            None => return Err(anyhow!("message channel closed unexpectedly")),
                        };
                        input_device.write_event(&msg);
                    }
                }
                Ok(())
            });
        }
        first_iteration_box = None;

        // spawn tasks for reading devices
        {
            let writer = writer.clone();
            let return_broadcast_rx = return_broadcast_rx.subscribe();
            task::spawn(async move {
                read_from_fd_runner(device, writer, return_broadcast_rx).await;
            });
        }
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
                    println!("sending broadcast");
                    broadcast.send(())?;
                    std::thread::sleep(time::Duration::from_secs(1));

                    println!("sending chan");
                    let (tx, rx) = oneshot::channel();
                    return_tx.send(tx).unwrap();
                    reader_rx_box = Some(rx.await.unwrap());
                    println!("cleanup done");
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
