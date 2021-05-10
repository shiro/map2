use crate::*;
use anyhow::Error;

#[derive(Debug)]
pub enum ExecutionMessage {
    EatEv(KeyAction),
    AddMapping(usize, KeyActionWithMods, Block, GuardedVarMap),
    GetFocusedWindowInfo(mpsc::Sender<Option<ActiveWindowInfo>>),
    RegisterWindowChangeCallback(Block, GuardedVarMap),
    Exit(i32),
    FatalError(Error, i32),
}

pub type ExecutionMessageSender = tokio::sync::mpsc::Sender<ExecutionMessage>;
