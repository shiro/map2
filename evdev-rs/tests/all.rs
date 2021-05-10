use evdev_rs::enums::*;
use evdev_rs::*;
use std::fs::File;
use std::os::unix::io::AsRawFd;

#[test]
fn context_create() {
    assert!(UninitDevice::new().is_some());
}

#[test]
fn context_create_with_file() {
    let f = File::open("/dev/input/event0").unwrap();
    let _d = Device::new_from_file(f).unwrap();
}

#[test]
fn context_set_file() {
    let d = UninitDevice::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();
    let _device = d.set_file(f).unwrap();
}

#[test]
fn context_change_file() {
    let d = UninitDevice::new().unwrap();
    let f1 = File::open("/dev/input/event0").unwrap();
    let f2 = File::open("/dev/input/event0").unwrap();
    let f2_fd = f2.as_raw_fd();

    let mut d = d.set_file(f1).unwrap();
    d.change_file(f2).unwrap();

    assert_eq!(d.file().as_raw_fd(), f2_fd);
}

#[test]
fn context_grab() {
    let d = UninitDevice::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    let mut d = d.set_file(f).unwrap();
    d.grab(GrabMode::Grab).unwrap();
    d.grab(GrabMode::Ungrab).unwrap();
}

#[test]
fn device_get_name() {
    let d = UninitDevice::new().unwrap();

    d.set_name("hello");
    assert_eq!(d.name().unwrap(), "hello");
}

#[test]
fn device_get_uniq() {
    let d = UninitDevice::new().unwrap();

    d.set_uniq("test");
    assert_eq!(d.uniq().unwrap(), "test");
}

#[test]
fn device_get_phys() {
    let d = UninitDevice::new().unwrap();

    d.set_phys("test");
    assert_eq!(d.phys().unwrap(), "test");
}

#[test]
fn device_get_product_id() {
    let d = UninitDevice::new().unwrap();

    d.set_product_id(5);
    assert_eq!(d.product_id(), 5);
}

#[test]
fn device_get_vendor_id() {
    let d = UninitDevice::new().unwrap();

    d.set_vendor_id(5);
    assert_eq!(d.vendor_id(), 5);
}

#[test]
fn device_get_bustype() {
    let d = UninitDevice::new().unwrap();

    d.set_bustype(5);
    assert_eq!(d.bustype(), 5);
}

#[test]
fn device_get_version() {
    let d = UninitDevice::new().unwrap();

    d.set_version(5);
    assert_eq!(d.version(), 5);
}

#[test]
fn device_get_absinfo() {
    let d = UninitDevice::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    let d = d.set_file(f).unwrap();
    for code in EventCode::EV_SYN(EV_SYN::SYN_REPORT).iter() {
        let absinfo: Option<AbsInfo> = d.abs_info(&code);

        match absinfo {
            None => ..,
            Some(_a) => ..,
        };
    }
}

#[test]
fn device_has_property() {
    let d = UninitDevice::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    let d = d.set_file(f).unwrap();
    for prop in InputProp::INPUT_PROP_POINTER.iter() {
        if d.has(&prop) {
            panic!("Prop {} is set, shouldn't be", prop);
        }
    }
}

#[test]
fn device_has_syn() {
    let d = UninitDevice::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    let d = d.set_file(f).unwrap();

    assert!(d.has(&EventType::EV_SYN)); // EV_SYN
    assert!(d.has(&EventCode::EV_SYN(EV_SYN::SYN_REPORT))); // SYN_REPORT
}

#[test]
fn device_get_value() {
    let d = UninitDevice::new().unwrap();
    let f = File::open("/dev/input/event0").unwrap();

    let d = d.set_file(f).unwrap();

    let v2 = d.event_value(&EventCode::EV_SYN(EV_SYN::SYN_REPORT)); // SYN_REPORT
    assert_eq!(v2, Some(0));
}

#[test]
fn check_event_name() {
    assert_eq!("EV_ABS", EventType::EV_ABS.to_string());
}

#[test]
fn test_timeval() {
    assert_eq!(TimeVal::new(1, 1_000_000), TimeVal::new(2, 0));
    assert_eq!(TimeVal::new(-1, -1_000_000), TimeVal::new(-2, 0));
    assert_eq!(TimeVal::new(1, -1_000_000), TimeVal::new(0, 0));
    assert_eq!(TimeVal::new(-100, 1_000_000 * 100), TimeVal::new(0, 0));
}
