use std::{io, time};
use std::collections::HashMap;
use std::fs::File;

use anyhow::{anyhow, bail, Result};
use evdev_rs::*;
use evdev_rs::enums::*;
use regex::Regex;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::Sender;
use tokio::task;
use walkdir::{DirEntry, WalkDir};
use crate::device::device_util;
use crate::log_msg;

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


async fn read_from_fd_runner(device: Device, reader_rx: mpsc::Sender<InputEvent>) {
    let mut a: io::Result<(ReadStatus, InputEvent)>;
    loop {
        a = device.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING);
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
                Some(libc::EAGAIN) => continue,
                _ => {
                    println!("{}", err);
                    break;
                }
            }
        }
    }
}


async fn runner(fd_pattens: Vec<Regex>, reader_init: oneshot::Sender<mpsc::Sender<InputEvent>>, mut writer: mpsc::Sender<InputEvent>) {
    // make empty map
    let mut fd_map = HashMap::new();

    // let mut writer_box = writer;
    let mut reader_init_box = Some(reader_init);
    let (reader_tx, reader_rx) = mpsc::channel(128);
    let mut reader_rx_box = Some(reader_rx);

    // TODO on change
    let mut ran = false;
    while ran == false {
        let input_fd_path_list = get_fd_list(&fd_pattens);
        if input_fd_path_list.len() < 1 {
            // out_ev_tx.send(Err(anyhow!("no matching fd found"))).await;
            break;
        }

        println!("{:?}", input_fd_path_list);


        let mut devices = vec![];
        for fd_path in input_fd_path_list {
            if fd_map.get(&fd_path).is_some() { continue; }
            fd_map.insert(fd_path.clone(), true);

            // grab fd_path as fd
            let fd_file = File::open(fd_path).unwrap();
            let mut device = Device::new_from_file(fd_file).unwrap();
            device.grab(GrabMode::Grab).unwrap();
            devices.push(device);
        }

        {
            let mut it = devices.iter_mut();
            let main_dev = it.next().unwrap();

            for device in it {
                device_util::clone_device(device, main_dev);
            }
        }


        for device in devices {
            // spawn virtual device and write to it
            if let Some(mut reader_rx) = reader_rx_box {
                let reader_init = reader_init_box.unwrap();

                // make uinput device on fd as fd_uinput
                let input_device = UInputDevice::create_from_device(&device).unwrap();

                // spawn writing task with fd_uinput, move rx
                // let reader = reader_rx.clone();
                task::spawn(async move {
                    loop {
                        let msg: InputEvent = match reader_rx.recv().await {
                            Some(v) => v,
                            None => break,
                        };
                        // print_event_debug(&msg);
                        // log_msg(&format!("{}, {}", &msg.event_code, &msg.value));
                        input_device.write_event(&msg);
                    }
                });

                // send the reader to the client
                reader_init.send(reader_tx.clone());
            }
            // unset moved values
            reader_init_box = None;
            reader_rx_box = None;

            // spawn tasks for reading devices
            {
                let writer = writer.clone();
                task::spawn(async move {
                    read_from_fd_runner(device, writer).await;
                });
            }
        }

        // tokio::time::sleep(time::Duration::from_millis(5000)).await;
        ran = true;
    }
}


pub(crate) async fn bind_udev_inputs(fd_patterns: Vec<&str>, reader_init_tx: oneshot::Sender<mpsc::Sender<InputEvent>>, mut writer_tx: mpsc::Sender<InputEvent>) -> Result<()> {
    if fd_patterns.len() < 1 { bail!(anyhow!("need at least 1 pattern")); }

    let fd_patterns_regex = fd_patterns.into_iter().map(|v| Regex::new(v).unwrap()).collect();

    task::spawn(async move {
        runner(fd_patterns_regex, reader_init_tx, writer_tx).await;
    });

    Ok(())
}
