use crate::*;
use messaging::*;

#[derive(Default)]
pub struct ScriptTestingParameters<'a> {
    pub script_path: &'a str,
}

pub struct ScriptTestingAPI {
    pub event_delay: Option<u64>,

    ev_reader_tx: mpsc::Sender<InputEvent>,
    ev_writer_rx: mpsc::Receiver<InputEvent>,
    stop_tx: futures_intrusive::channel::shared::Sender<()>,
    stdout: Arc<tokio::sync::Mutex<Vec<u8>>>,
}

impl ScriptTestingAPI {
    pub async fn stop(&mut self) {
        if let Some(delay) = self.event_delay {
            sleep(delay);
        }

        let _ = self.stop_tx.send(()).await;
    }

    pub async fn write_event(&mut self, ev: InputEvent) -> Result<()> {
        if let Some(delay) = self.event_delay {
            sleep(delay);
        }

        self.ev_reader_tx.send(ev).await?;
        Ok(())
    }

    pub async fn write_action(&mut self, action: KeyAction) -> Result<()> {
        self.write_event(action.to_input_ev()).await
    }

    pub async fn collect_output_ev(&mut self) -> Vec<InputEvent> {
        let mut vec = vec![];
        while let Ok(ev) = self.ev_writer_rx.try_recv() {
            vec.push(ev);
        }
        vec
    }

    #[allow(unused)]
    pub async fn collect_stdout(&mut self) -> String {
        let result = String::from_utf8_lossy(&self.stdout.lock().await).into_owned();
        self.reset_stdout().await;
        result
    }

    #[allow(unused)]
    pub async fn reset_stdout(&mut self) { self.stdout.lock().await.clear(); }
}

pub async fn test_script(
    parameters: ScriptTestingParameters<'_>,
) -> Result<ScriptTestingAPI> {
    let mut script_file = fs::File::open(parameters.script_path)?;

    let script_ast = script::parse_script(&mut script_file);

    let mut state = State::new();
    let window_cycle_token: usize = 0;
    let mut mappings = CompiledKeyMappings::new();
    let mut window_change_handlers = vec![];
    let stdout = Arc::new(tokio::sync::Mutex::new(vec![]));

    let (execution_message_tx, mut execution_message_rx) = mpsc::channel(128);
    let (ev_reader_tx, mut ev_reader_rx) = mpsc::channel(128);
    let (mut ev_writer_tx, ev_writer_rx) = mpsc::channel(128);

    let (stop_tx, stop_rx) = futures_intrusive::channel::shared::unbuffered_channel();
    {
        let mut execution_message_tx = execution_message_tx.clone();
        let stdout = stdout.clone();
        task::spawn(async move {
            loop {
                tokio::select! {
                        Some(ev) = ev_reader_rx.recv() => {
                            event_handlers::handle_stdin_ev(&mut state, ev, &mut mappings,
                                &mut ev_writer_tx, &mut execution_message_tx, window_cycle_token).await.unwrap();
                        }
                        Some(msg) = execution_message_rx.recv() => {
                            // don't terminate during testing
                            if let ExecutionMessage::Exit(_) = msg{ return; }

                            event_handlers::handle_execution_message(&mut *stdout.lock().await, window_cycle_token, msg, &mut state,
                                &mut mappings, &mut window_change_handlers).await;
                        }
                        Some(_) = stop_rx.receive() => {
                            return;
                        }
                    }
            }
        });
    }

    script::evaluate_script(script_ast, execution_message_tx, ev_reader_tx.clone(), 0).await;

    let api = ScriptTestingAPI {
        ev_reader_tx,
        ev_writer_rx,
        stop_tx,
        stdout,
        event_delay: None,
    };

    Ok(api)
}


pub fn sleep(duration: u64) {
    std::thread::sleep(time::Duration::from_millis(duration));
}