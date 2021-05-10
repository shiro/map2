use evdev_rs::enums::*;
use evdev_rs::*;
use std::fs::File;
use std::io;

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

fn print_sync_dropped_event(ev: &InputEvent) {
    print!("SYNC DROPPED: ");
    print_event(ev);
}

fn main() {
    let mut args = std::env::args();

    if args.len() != 2 {
        usage();
        std::process::exit(1);
    }

    let path = &args.nth(1).unwrap();
    let f = File::open(path).unwrap();

    let u_d = UninitDevice::new().unwrap();
    let d = u_d.set_file(f).unwrap();

    println!(
        "Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
        d.bustype(),
        d.vendor_id(),
        d.product_id()
    );
    println!("Evdev version: {:x}", d.driver_version());
    println!("Input device name: \"{}\"", d.name().unwrap_or(""));
    println!("Phys location: {}", d.phys().unwrap_or(""));
    println!("Uniq identifier: {}", d.uniq().unwrap_or(""));

    print_bits(&d);
    print_props(&d);

    let mut a: io::Result<(ReadStatus, InputEvent)>;
    loop {
        a = d.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING);
        if a.is_ok() {
            let mut result = a.ok().unwrap();
            match result.0 {
                ReadStatus::Sync => {
                    println!("::::::::::::::::::::: dropped ::::::::::::::::::::::");
                    while result.0 == ReadStatus::Sync {
                        print_sync_dropped_event(&result.1);
                        a = d.next_event(ReadFlag::SYNC);
                        if a.is_ok() {
                            result = a.ok().unwrap();
                        } else {
                            break;
                        }
                    }
                    println!("::::::::::::::::::::: re-synced ::::::::::::::::::::");
                }
                ReadStatus::Success => print_event(&result.1),
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
