use anyhow::Error;

use crate::*;

#[derive(Debug)]
pub enum ExecutionMessage {
    // EatEv(KeyAction),
    AddMapping(usize, KeyActionWithMods, Block, GuardedVarMap),
    GetFocusedWindowInfo(mpsc::Sender<Option<ActiveWindowInfo>>),
    RegisterWindowChangeCallback(Block, GuardedVarMap),
    Write(String),
    Exit(i32),
    FatalError(Error, i32),
}

pub type ExecutionMessageSender = tokio::sync::mpsc::Sender<ExecutionMessage>;
