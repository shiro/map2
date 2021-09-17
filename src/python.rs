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

struct InstanceHandleSharedState {
    mappings: HashMap<KeyActionWithMods, RuntimeAction>,
}

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


fn map_action_to_seq(from: KeyActionWithMods, to: Vec<ParsedKeyAction>) -> Mapping {
    let mut seq = to.to_key_actions()
        .into_iter()
        .map(|action| RuntimeKeyAction::KeyAction(action))
        .collect();

    (from, RuntimeAction::ActionSequence(seq))
}

fn map_action_to_click(from: &KeyActionWithMods, to: &KeyClickActionWithMods) -> Mapping {
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

fn map_action_to_action(from: &KeyActionWithMods, to: &KeyActionWithMods) -> Mapping {
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

fn map_click_to_seq(from: KeyClickActionWithMods, to: Vec<ParsedKeyAction>) -> [Mapping; 3] {
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

fn map_click_to_click(from: &KeyClickActionWithMods, to: &KeyClickActionWithMods) -> [Mapping; 3] {
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

fn map_click_to_action(from: &KeyClickActionWithMods, to: &KeyActionWithMods) -> [Mapping; 3] {
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

#[pymethods]
impl InstanceHandle {
    pub fn send(&mut self, val: String) {
        let actions = parse_key_sequence_py(val.as_str()).unwrap();

        for action in actions.to_key_actions() {
            futures::executor::block_on(
                self.ev_tx.send(action.to_input_ev())
            ).unwrap();
            futures::executor::block_on(
                self.ev_tx.send(SYN_REPORT.clone())
            ).unwrap();
        }
    }
    pub fn send_modifier(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence_py(val.as_str())
            .unwrap()
            .to_key_actions();

        if actions.len() != 1 {
            return Err(PyValueError::new_err(format!("expected a single key action, got {}", actions.len())));
        }

        let action = actions.get(0).unwrap();

        if [*KEY_LEFT_CTRL, *KEY_RIGHT_CTRL, *KEY_LEFT_ALT, *KEY_RIGHT_ALT, *KEY_LEFT_SHIFT, *KEY_RIGHT_SHIFT, *KEY_LEFT_META, *KEY_RIGHT_META]
            .contains(&action.key) {
            self.message_tx.send(ControlMessage::UpdateModifiers(*action));
        } else {
            return Err(PyValueError::new_err("key action needs to be a modifier event"));
        }

        futures::executor::block_on(self.ev_tx.send(action.to_input_ev())).unwrap();
        futures::executor::block_on(self.ev_tx.send(SYN_REPORT.clone())).unwrap();
        Ok(())
    }

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        if let Ok(to) = to.extract::<String>(py) {
            self._map_internal(from, to);
            return Ok(());
        }

        let is_callable = to.cast_as::<PyAny>(py)
            .map_or(false, |obj| obj.is_callable());

        if is_callable {
            self._map_callback(from, to);
            return Ok(());
        }

        return Err(PyTypeError::new_err("unknown type"));
    }

    fn _map_callback(&mut self, from: String, to: PyObject) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();

        match from {
            ParsedKeyAction::KeyAction(from) => {
                self.message_tx.send(ControlMessage::AddMapping(from, RuntimeAction::PythonCallback(to)));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(1), RuntimeAction::PythonCallback(to)));
                self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(0), RuntimeAction::NOP));
                self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(2), RuntimeAction::NOP));
            }
        }

        Ok(())
    }

    fn _map_internal(&mut self, from: String, to: String) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();
        let mut to = parse_key_sequence_py(&to).unwrap();

        match from {
            ParsedKeyAction::KeyAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    // action to click
                    if let ParsedKeyAction::KeyClickAction(to) = to {
                        let mapping = map_action_to_click(&from, &to);
                        self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
                        return Ok(());
                    }
                    // action to action
                    if let ParsedKeyAction::KeyAction(to) = to {
                        let mapping = map_action_to_action(&from, &to);
                        self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
                        return Ok(());
                    }
                }

                // action to seq
                let mapping = map_action_to_seq(from, to);
                self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    match to.remove(0) {
                        // click to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mappings = map_click_to_click(&from, &to);
                            IntoIter::new(mappings).for_each(|(from, to)| {
                                self.message_tx.send(ControlMessage::AddMapping(from, to));
                            });
                            return Ok(());
                        }
                        // click to action
                        ParsedKeyAction::KeyAction(to) => {
                            let mappings = map_click_to_action(&from, &to);
                            IntoIter::new(mappings).for_each(|(from, to)| {
                                self.message_tx.send(ControlMessage::AddMapping(from, to));
                            });
                            return Ok(());
                        }
                    };
                }

                // click to seq
                let mappings = map_click_to_seq(from, to);
                IntoIter::new(mappings).for_each(|(from, to)| {
                    self.message_tx.send(ControlMessage::AddMapping(from, to));
                });
            }
        }

        Ok(())
    }
}


#[pyfunction]
fn setup(py: Python, callback: PyObject) -> PyResult<InstanceHandle> {
    let handle = _setup(callback).unwrap();
    Ok(handle)
}

#[pymodule]
fn map2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(setup, m)?)?;

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