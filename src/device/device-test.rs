#[macro_use]
extern crate lazy_static;
extern crate regex;

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

fn print_event(ev: &InputEvent) {
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

#[tokio::main]
async fn main() {
    let patterns = vec![
        "/dev/input/by-id/.*-event-mouse",
        "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd"
    ];

    let (mut reader_init_tx, mut reader_init_rx) = oneshot::channel();
    let (mut writer_tx, mut writer_rx) = mpsc::channel(128);

    // start coroutine
    dev(patterns, reader_init_tx, writer_tx).await;

    let reader_tx = reader_init_rx.await.unwrap();

    loop {
        let ev = writer_rx.recv().await.unwrap();
        // println!("wow");
        print_event(&ev);
        reader_tx.send(ev).await;
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


// oneshot<Sender<Ev>>
async fn runner(fd_pattens: Vec<Regex>, reader_init: oneshot::Sender<mpsc::Sender<InputEvent>>, mut writer: mpsc::Sender<InputEvent>) {
    // make empty map
    let mut fd_map = HashMap::new();

    // let mut writer_box = writer;
    let mut reader_init_box = Some(reader_init);
    let (reader_tx, reader_rx) = mpsc::channel(128);
    let mut reader_rx_box = Some(reader_rx);


    // on change
    loop {
        // list = fs_list()
        let input_fd_path_list = get_fd_list(&fd_pattens);
        if input_fd_path_list.len() < 1 {
            // out_ev_tx.send(Err(anyhow!("no matching fd found"))).await;
            break;
        }

        println!("{:?}", input_fd_path_list);

        // for fd_path in list
        for fd_path in input_fd_path_list {
            // if fd_path in map, continue
            if fd_map.get(&fd_path).is_some() { continue; }
            // add fd_path to the map
            fd_map.insert(fd_path.clone(), true);

            // grab fd_path as fd
            let fd_file = File::open(fd_path).unwrap();
            let mut device = Device::new_from_file(fd_file).unwrap();
            device.grab(GrabMode::Grab).unwrap();


            // spawn virtual device and write to it
            if let Some(mut reader_rx) = reader_rx_box {
                let reader_init = reader_init_box.unwrap();

                // make uinput device on fd as fd_uinput
                let input_device = UInputDevice::create_from_device(&device).unwrap();

                // spawn writing task with fd_uinput, move rx
                // let reader = reader_rx.clone();
                task::spawn(async move {
                    loop {
                        let msg = match reader_rx.recv().await {
                            Some(v) => v,
                            None => break,
                        };
                        input_device.write_event(&msg);
                    }
                });

                // oneshot.send tx
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


        tokio::time::sleep(time::Duration::from_millis(5000)).await;
    }
}


async fn dev(fd_patterns: Vec<&str>, reader_init_tx: oneshot::Sender<mpsc::Sender<InputEvent>>, mut writer_tx: mpsc::Sender<InputEvent>) -> Result<()> {
    if fd_patterns.len() < 1 { bail!(anyhow!("need at least 1 pattern")); }

    let fd_patterns_regex = fd_patterns.into_iter().map(|v| Regex::new(v).unwrap()).collect();

    task::spawn(async move {
        runner(fd_patterns_regex, reader_init_tx, writer_tx).await;
    });

    Ok(())


    // let fd_list = get_fd_list();
    // if fd_list.len() < 1 { return Err(anyhow!("no matching fd found")); }


    // let first_fd_path = fd_list.remove(0);

    // let f = File::open(first_fd_path).unwrap();

    // let mut device = Device::new_from_file(f).unwrap();
    // device.grab(GrabMode::Grab).unwrap();
    //
    //
    // let (tx, rx) = mpsc::channel(128);
    //
    //
    // let input_device = UInputDevice::create_from_device(&device).unwrap();
    //
    //
    // println!("name: {:?}", device.name());
    //
    //
    // println!(
    //     "Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
    //     device.bustype(),
    //     device.vendor_id(),
    //     device.product_id()
    // );
    //
    // print_bits(&device);
    // print_props(&device);
    //
    // let mut a: io::Result<(ReadStatus, InputEvent)>;
    // loop {
    //     a = device.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING);
    //     if a.is_ok() {
    //         let mut result = a.ok().unwrap();
    //         match result.0 {
    //             ReadStatus::Sync => { // dropped, need to sync
    //                 while result.0 == ReadStatus::Sync {
    //                     a = device.next_event(ReadFlag::SYNC);
    //                     if a.is_ok() {
    //                         result = a.ok().unwrap();
    //                     } else { // something failed, abort sync and carry on
    //                         break;
    //                     }
    //                 }
    //             }
    //             ReadStatus::Success => {
    //                 print_event(&result.1);
    //                 input_device.write_event(&result.1).unwrap();
    //             }
    //         }
    //     } else {
    //         let err = a.err().unwrap();
    //         match err.raw_os_error() {
    //             Some(libc::EAGAIN) => continue,
    //             _ => {
    //                 println!("{}", err);
    //                 break;
    //             }
    //         }
    //     }
    // }
}
