use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use evdev_rs::*;
use evdev_rs::Device;
use evdev_rs::enums::*;
use regex::Regex;
use tokio::sync::{mpsc, oneshot};
use tokio::task;
use walkdir::{DirEntry, WalkDir};

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

fn clone_code_bits(src: &Device, dst: &mut Device, ev_code: &EventCode, max: &EventCode) -> Result<()>{
    for code in ev_code.iter() {
        if code == *max {
            break;
        }
        if !src.has(&code) {
            continue;
        }

        dst.enable(&code)?;
        // println!("    Event code: {}", code);
        // match code {
        //     EventCode::EV_ABS(k) => print_abs_bits(src, &k),
        //     // _ => (),
        // }
    }
    Ok(())
}

fn clone_bits(src: &Device, dst: &mut Device) -> Result<()>{
    // println!("Supported events:");
    for ev_type in EventType::EV_SYN.iter() {
        match ev_type {
            EventType::EV_KEY => clone_code_bits(
                src,
                dst,
                &EventCode::EV_KEY(EV_KEY::KEY_RESERVED),
                &EventCode::EV_KEY(EV_KEY::KEY_MAX),
            )?,
            EventType::EV_REL => clone_code_bits(
                src,
                dst,
                &EventCode::EV_REL(EV_REL::REL_X),
                &EventCode::EV_REL(EV_REL::REL_MAX),
            )?,
            EventType::EV_ABS => clone_code_bits(
                src,
                dst,
                &EventCode::EV_ABS(EV_ABS::ABS_X),
                &EventCode::EV_ABS(EV_ABS::ABS_MAX),
            )?,
            EventType::EV_LED => clone_code_bits(
                src,
                dst,
                &EventCode::EV_LED(EV_LED::LED_NUML),
                &EventCode::EV_LED(EV_LED::LED_MAX),
            )?,
            _ => (),
        }
    }
    Ok(())
}

fn clone_props(src: &Device, dst: &mut Device) -> Result<()> {
    for input_prop in InputProp::INPUT_PROP_POINTER.iter() {
        if src.has(&input_prop) {
            dst.enable(&input_prop)?;
        }
    }
    Ok(())
}

pub(crate) fn clone_device(src: &Device, dst: &mut Device) -> Result<()>{
    clone_props(src, dst)?;
    clone_bits(src, dst)?;
    Ok(())
}