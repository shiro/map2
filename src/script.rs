use crate::*;
use std::io::{Read, Seek, SeekFrom};
use crate::messaging::ExecutionMessage;


pub fn parse_script(script_file: &mut fs::File) -> Block {
    let script_file_length = script_file.seek(SeekFrom::End(0))
        .map_err(|err| anyhow!("failed seek operation on script file: {}", err))
        .unwrap();

    // restore head
    script_file.seek(SeekFrom::Start(0))
        .map_err(|err| anyhow!("failed seek operation on script file: {}", err))
        .unwrap();

    let mut raw = String::with_capacity(script_file_length as usize);
    script_file.read_to_string(&mut raw)
        .map_err(|err| anyhow!("failed to read script file: {}", err))
        .unwrap();

    let global = parsing::parser::parse_script(&*raw).unwrap();

    global
}


pub async fn evaluate_script(
    script_ast: Block,
    mut execution_message_tx: mpsc::Sender<ExecutionMessage>,
    ev_reader_tx: mpsc::Sender<InputEvent>,
    window_cycle_token: usize,
) {
    let mut amb = Ambient {
        ev_writer_tx: ev_reader_tx,
        window_cycle_token,
        message_tx: Some(&mut execution_message_tx),
        modifier_state: &KeyModifierState::new(),
    };

    eval_block(&script_ast, &mut GuardedVarMap::new(Mutex::new(VarMap::new(None))), &mut amb).await;
}
