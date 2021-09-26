use std::thread;
use std::array::IntoIter;

use evdev_rs::enums::{EV_KEY, EventType};
use evdev_rs::TimeVal;
use pyo3::prelude::*;

use crate::{EventCode, INPUT_EV_DUMMY_TIME, InputEvent};
use crate::*;
use crate::task::JoinHandle;
use anyhow::Error;
use crate::device::device_logging::print_event_debug;
use crate::parsing::python::{parse_key_action_with_mods_py, parse_key_sequence_py};
use crate::parsing::parser::parse_key_sequence;
use crate::parsing::key_action::*;
use crate::ignore_list::IgnoreList;
use pyo3::exceptions;
use pyo3::exceptions::{PyTypeError, PyValueError};
use std::collections::HashSet;
use std::time::Duration;
use crate::python_window::Window;

#[pyclass]
struct PyKey {
    #[pyo3(get, set)]
    code: u32,
    #[pyo3(get, set)]
    value: i32,
}


#[pyclass]
struct InstanceHandle {
    exit_tx: oneshot::Sender<()>,
    join_handle: std::thread::JoinHandle<()>,
    ev_tx: mpsc::Sender<InputEvent>,
    message_tx: mpsc::UnboundedSender<ControlMessage>,
}

pub type Mapping = (KeyActionWithMods, RuntimeAction);
pub type Mappings = HashMap<KeyActionWithMods, RuntimeAction>;


impl InstanceHandle {
    pub fn new(exit_tx: oneshot::Sender<()>, join_handle: std::thread::JoinHandle<()>,
               ev_tx: mpsc::Sender<InputEvent>,
               message_tx: mpsc::UnboundedSender<ControlMessage>) -> Self {
        InstanceHandle {
            exit_tx,
            join_handle,
            ev_tx,
            message_tx,
        }
    }
}

#[derive(Debug)]
pub enum RuntimeKeyAction {
    KeyAction(KeyAction),
    ReleaseRestoreModifiers(KeyModifierFlags, KeyModifierFlags, i32),
}

#[derive(Debug)]
pub enum RuntimeAction {
    ActionSequence(Vec<RuntimeKeyAction>),
    PythonCallback(PyObject),
    NOP,
}


pub fn map_action_to_seq(from: KeyActionWithMods, to: Vec<ParsedKeyAction>) -> Mapping {
    let mut seq = to.to_key_actions()
        .into_iter()
        .map(|action| RuntimeKeyAction::KeyAction(action))
        .collect();

    (from, RuntimeAction::ActionSequence(seq))
}

pub fn map_action_to_click(from: &KeyActionWithMods, to: &KeyClickActionWithMods) -> Mapping {
    let mut seq = vec![];
    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

    seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));
    seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));

    // revert to original
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

    (from.clone(), RuntimeAction::ActionSequence(seq))
}

pub fn map_action_to_action(from: &KeyActionWithMods, to: &KeyActionWithMods) -> Mapping {
    let mut seq = vec![];
    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

    seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: to.value }));

    // revert to original
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

    (from.clone(), RuntimeAction::ActionSequence(seq))
}

pub fn map_click_to_seq(from: KeyClickActionWithMods, to: Vec<ParsedKeyAction>) -> [Mapping; 3] {
    let mut seq = to.to_key_actions()
        .into_iter()
        .map(|action| RuntimeKeyAction::KeyAction(action))
        .collect();

    let down_mapping = (KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    // stub up and repeat, click only triggers sequence on down press
    let up_mapping = (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    let repeat_mapping = (KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    [down_mapping, up_mapping, repeat_mapping]
}

pub fn map_click_to_click(from: &KeyClickActionWithMods, to: &KeyClickActionWithMods) -> [Mapping; 3] {
    let mut down_mapping;
    {
        let mut seq = vec![];
        seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }
        seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));
        down_mapping = (KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    }
    let mut up_mapping;
    {
        let mut seq = vec![];
        seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }
        seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));
        up_mapping = (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    }
    let mut repeat_mapping;
    {
        let mut seq = vec![];
        seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_REPEAT }));
        repeat_mapping = (KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    }
    [down_mapping, up_mapping, repeat_mapping]
}

pub fn map_click_to_action(from: &KeyClickActionWithMods, to: &KeyActionWithMods) -> [Mapping; 3] {
    let mut seq = vec![];

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

    seq.push(RuntimeKeyAction::KeyAction((KeyAction { key: to.key, value: to.value })));

    // revert to original
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

    let down_mapping = (KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    // stub up and repeat, click only triggers action on down press
    let up_mapping = (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    let repeat_mapping = (KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    [down_mapping, up_mapping, repeat_mapping]
}

#[pyfunction]
fn setup(py: Python, callback: PyObject) -> PyResult<InstanceHandle> {
    let handle = _setup(callback).unwrap();
    Ok(handle)
}

#[pymodule]
fn map2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(setup, m)?)?;
    m.add_class::<Window>()?;

    Ok(())
}

#[derive(Debug)]
pub enum ControlMessage {
    AddMapping(KeyActionWithMods, RuntimeAction),
    UpdateModifiers(KeyAction),
}

fn _setup(callback: PyObject) -> Result<InstanceHandle> {
    let (mut control_tx, mut control_rx) = mpsc::unbounded_channel();

    let (exit_tx, exit_rx) = oneshot::channel();

    // TODO move out device initialization (which requires a scheduler to spawn tasks) elsewhere, don't use oneshot for this
    let (_ev_tx_init_tx, _ev_tx_init_rx) = oneshot::channel();

    let join_handle = thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let mut configuration = parse_cli().unwrap();

            // initialize global state
            // let mut stdout = io::stdout();
            let mut window_cycle_token: usize = 0;
            // let mut mappings = CompiledKeyMappings::new();
            // let mut window_change_handlers = vec![];

            // initialize device communication channels
            let (ev_writer_tx, mut ev_writer_rx) = mpsc::channel(128);
            let (ev_reader_init_tx, ev_reader_init_rx) = oneshot::channel();

            bind_udev_inputs(&configuration.devices, ev_reader_init_tx, ev_writer_tx).await?;
            let mut ev_reader_tx = ev_reader_init_rx.await?;

            _ev_tx_init_tx.send(ev_reader_tx.clone()).unwrap();

            let mut state = State::new();
            let mut mappings = Mappings::new();

            loop {
                tokio::select! {
                    Some(ev) = ev_writer_rx.recv() => {
                        event_handlers::handle_stdin_ev(&mut state, ev, &mappings, &mut ev_reader_tx).await.unwrap();
                    }
                    Some(msg) = control_rx.recv() => {
                        // println!("{:?}", &msg);
                        event_handlers::handle_control_message(window_cycle_token, msg, &mut state, &mut mappings).await;
                    }
                }
            }

            // exit_rx.await?;
            #[allow(unreachable_code)]
                Ok::<(), anyhow::Error>(())
        }).unwrap();
    });


    let _ev_tx = futures::executor::block_on(_ev_tx_init_rx).unwrap();

    let handle = InstanceHandle::new(exit_tx, join_handle, _ev_tx, control_tx);

    Ok(handle)
}

