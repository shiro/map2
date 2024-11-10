use crate::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use std::os::fd::AsFd;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use pyo3::types::PyDict;
use pyo3::{pyclass, pymethods, PyResult};
use tempfile::tempfile;
use unicode_segmentation::UnicodeSegmentation;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_keyboard::KeymapFormat::XkbV1;
use wayland_client::protocol::wl_seat;
use wayland_client::{protocol::wl_registry, Dispatch, QueueHandle};
use wayland_client::{Connection, EventQueue, Proxy};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_manager_v1;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;
use wl_seat::WlSeat;
use xkeysym::Keysym;
use zwp_virtual_keyboard_manager_v1::Event as KeyboardManagerEvent;

use crate::encoding;

#[pyclass]
pub struct VirtualWriter {
    keyboard: VirtualKeyboard,
}

#[pymethods]
impl VirtualWriter {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(kwargs: Option<PyBound<PyDict>>) -> PyResult<Self> {
        let keyboard = VirtualKeyboard::new().unwrap();
        Ok(Self { keyboard })
    }

    pub fn send(&mut self, val: String) {
        self.keyboard.send(&val);
    }
}

struct AppData;

#[derive(Debug, Clone, Copy)]
enum KeyState {
    Pressed = 1,
    Released = 0,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for AppData {
    fn event(
        state: &mut AppData,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        data: &GlobalListContents,
        conn: &Connection,
        qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<WlSeat, ()> for AppData {
    fn event(
        state: &mut AppData,
        proxy: &WlSeat,
        event: wl_seat::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for AppData {
    fn event(
        state: &mut AppData,
        proxy: &ZwpVirtualKeyboardManagerV1,
        event: KeyboardManagerEvent,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1, ()> for AppData {
    fn event(
        state: &mut AppData,
        proxy: &zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
        event: zwp_virtual_keyboard_v1::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<AppData>,
    ) {
    }
}

struct VirtualKeyboard {
    con: Connection,
    keybaord_manager: ZwpVirtualKeyboardManagerV1,
    keyboard: ZwpVirtualKeyboardV1,
    event_queue: EventQueue<AppData>,
}

impl VirtualKeyboard {
    pub fn new() -> Result<Self> {
        let con = Connection::connect_to_env().unwrap();
        let (globals, mut event_queue) = registry_queue_init::<AppData>(&con).unwrap();

        event_queue.roundtrip(&mut AppData).unwrap();

        let seat: WlSeat = globals.bind(&event_queue.handle(), 7..=8, ()).unwrap();
        let keybaord_manager: ZwpVirtualKeyboardManagerV1 = globals.bind(&event_queue.handle(), 1..=1, ()).unwrap();
        let mut keyboard = keybaord_manager.create_virtual_keyboard(&seat, &event_queue.handle(), ());

        event_queue.roundtrip(&mut AppData).unwrap();

        Ok(Self { con, keyboard, keybaord_manager, event_queue })
    }

    pub fn send(&mut self, input: &str) {
        let mut keymap: HashMap<Keysym, u32> = HashMap::new();

        for x in input.graphemes(true) {
            let encoded = x.chars().next().unwrap() as u32;
            let out = encoding::xkb_utf32_to_keysym(encoded);
            let idx = keymap.len() as u32;

            keymap.entry(out).or_insert(idx);
        }

        init_virtual_keyboard(&self.keyboard, &keymap).unwrap();

        for x in input.graphemes(true) {
            let encoded = x.chars().next().unwrap() as u32;
            let out = encoding::xkb_utf32_to_keysym(encoded);

            let key_idx = keymap.get(&out).unwrap() + 1;

            self.keyboard.key(0, key_idx, KeyState::Pressed as u32);
            self.event_queue.roundtrip(&mut AppData).unwrap();

            self.keyboard.key(0, key_idx, KeyState::Released as u32);
            self.event_queue.roundtrip(&mut AppData).unwrap();
        }
    }
}

struct KeymapEntry {
    keysym: Keysym,
}

fn init_virtual_keyboard(keyboard: &ZwpVirtualKeyboardV1, keymap: &HashMap<Keysym, u32>) -> Result<()> {
    let mut keymap_file = tempfile().map_err(|_| anyhow!("unable to create temporary file"))?;

    keymap_file.write_all("xkb_keymap {\n".as_bytes())?;
    keymap_file.write_all("xkb_keycodes \"(unnamed)\" {\n".as_bytes())?;
    keymap_file.write_all("minimum = 8;\n".as_bytes())?;
    keymap_file.write_all(format!("maximum = {};\n", keymap.len() + 8 + 1).as_bytes())?;

    for i in 0..keymap.len() {
        keymap_file.write_all(format!("<K{}> = {};\n", i + 1, i + 8 + 1).as_bytes())?;
    }

    keymap_file.write_all("};\n".as_bytes())?;

    keymap_file.write_all("xkb_types \"(unnamed)\" { include \"complete\" };\n".as_bytes())?;
    keymap_file.write_all("xkb_compatibility \"(unnamed)\" { include \"complete\" };\n".as_bytes())?;

    keymap_file.write_all("xkb_symbols \"(unnamed)\" {\n".as_bytes())?;

    for (key, idx) in keymap.iter().sorted() {
        keymap_file.write_all(format!("key <K{}> {{[", idx + 1).as_bytes())?;

        let alias = if let Some(name) = key.name() {
            name[3..name.len()].to_string()
        } else {
            format!("U{:X}", key.raw() & 0xFFFFF).to_string()
        };
        keymap_file.write_all(alias.as_bytes())?;

        keymap_file.write_all("]};\n".as_bytes())?;
    }

    keymap_file.write_all("};\n".as_bytes())?;
    keymap_file.write_all("};\n".as_bytes())?;
    keymap_file.flush()?;

    keymap_file.seek(SeekFrom::Start(0))?;

    let keymap_fd = keymap_file.as_fd();
    let keymap_size: u32 = keymap_file.metadata().unwrap().len().try_into()?;

    keyboard.keymap(XkbV1 as u32, keymap_fd, keymap_size);

    Ok(())
}
