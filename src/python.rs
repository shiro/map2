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
    message_tx: mpsc::UnboundedSender<ControlMessage>,
}

pub type Mapping = (KeyActionWithMods, RuntimeAction);
pub type Mappings = HashMap<KeyActionWithMods, RuntimeAction>;

struct InstanceHandleSharedState {
    mappings: HashMap<KeyActionWithMods, RuntimeAction>,
}

impl InstanceHandle {
    pub fn new(exit_tx: oneshot::Sender<()>, join_handle: std::thread::JoinHandle<()>, message_tx: mpsc::UnboundedSender<ControlMessage>) -> Self {
        InstanceHandle {
            exit_tx,
            join_handle,
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
    // PythonBlock(),
}


fn map_click_to_click(from: &KeyClickActionWithMods, to: &KeyClickActionWithMods) -> [Mapping; 3] {
    let mut down_mapping;
    {
        let mut seq = vec![];
        // seq.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));
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

#[pymethods]
impl InstanceHandle {
    pub fn map(&mut self, from: String, to: String) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();
        let mut to = parse_key_sequence_py(&to).unwrap();

        match from {
            ParsedKeyAction::KeyAction(action) => {
                unimplemented!();
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    // click to click
                    if let ParsedKeyAction::KeyClickAction(to) = to {
                        let mappings = map_click_to_click(&from, &to);
                        IntoIter::new(mappings).for_each(|(from, to)| {
                            self.message_tx.send(ControlMessage::AddMapping(from, to));
                        });
                        return Ok(());
                    }
                    // click to action
                    // if let ParsedKeyAction::KeyAction(to) = to.remove(0) {
                    //     unimplemented!();
                    //     // return Ok((next, (Expr::map_key_click_action(from, to), None)));
                    // }
                }
                unimplemented!();
            }
        }

        // let from = KeyActionWithMods::new(Key::from_str(&EventType::EV_KEY, "KEY_A").unwrap(), 0, KeyModifierFlags::new());
        // self.message_tx.send(ControlMessage::AddMapping(from, vec![]));
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
}

fn _setup(callback: PyObject) -> Result<InstanceHandle> {
    let (mut control_tx, mut control_rx) = mpsc::unbounded_channel();

    let (exit_tx, exit_rx) = oneshot::channel();
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
            let (ev_reader_init_tx, ev_reader_init_rx) = oneshot::channel();
            let (ev_writer_tx, mut ev_writer_rx) = mpsc::channel(128);

            bind_udev_inputs(&configuration.devices, ev_reader_init_tx, ev_writer_tx).await?;
            let mut ev_reader_tx = ev_reader_init_rx.await?;

            let mut state = State::new();
            let mut mappings = Mappings::new();

            loop {
                tokio::select! {
                    Some(ev) = ev_writer_rx.recv() => {
                        event_handlers::handle_stdin_ev(&mut state, ev, &mappings, &mut ev_reader_tx).await.unwrap();
                    }
                    Some(msg) = control_rx.recv() => {
                        // println!("{:?}", &msg);
                        event_handlers::handle_control_message(window_cycle_token, msg, &mut state, &mut mappings ).await;
                    }
                }

                // let code = match ev.event_code {
                //     EventCode::EV_KEY(code) => code,
                //     _ => continue,
                // };
                //
                //
                // let key = PyKey { code: code as u32, value: ev.value };
                // {
                //     use std::time::Instant;
                //     let now = Instant::now();
                //     let gil = Python::acquire_gil();
                //
                //
                //     let py = gil.python();
                //
                //     callback.call(py, (key, ), None);
                //
                //     let elapsed = now.elapsed();
                //     println!("Elapsed: {:.2?}", elapsed);
                // }
            }

            // exit_rx.await?;
            #[allow(unreachable_code)]
                Ok::<(), anyhow::Error>(())
        }).unwrap();
    });

    let handle = InstanceHandle::new(exit_tx, join_handle, control_tx);

    Ok(handle)
}