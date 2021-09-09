use map2::*;
use std::ops::Deref;
use std::thread;

mod event_handlers;

#[tokio::main]
async fn main() -> Result<()> {
    let mut configuration = parse_cli()?;

    // create X11 communication channels
    // let (window_ev_tx, mut window_ev_rx) = mpsc::channel(128);
    // let (mut execution_message_tx, mut message_rx) = mpsc::channel(128);

    // spawn X11 thread
    // tokio::spawn(async move {
    //     let x11_state = Arc::new(x11_initialize().unwrap());
    //
    //     loop {
    //         let x11_state_clone = x11_state.clone();
    //         let res = task::spawn_blocking(move || {
    //             get_window_info_x11(&x11_state_clone)
    //         }).await.unwrap();
    //
    //         if let Ok(Some(val)) = res {
    //             window_ev_tx.send(val).await.unwrap_or_else(|_| panic!());
    //         }
    //     }
    // });

    // initialize global state
    let mut stdout = io::stdout();
    let mut state = State::new();
    let mut window_cycle_token: usize = 0;
    let mut mappings = CompiledKeyMappings::new();
    let mut window_change_handlers = vec![];

    let script_ast = script::parse_script(&mut configuration.script_file);

    // add a small delay if run from TTY so we don't miss 'enter up' which is often released when the device is grabbed
    if atty::is(atty::Stream::Stdout) {
        thread::sleep(time::Duration::from_millis(300));
    }

    // initialize device communication channels
    let (ev_reader_init_tx, ev_reader_init_rx) = oneshot::channel();
    let (ev_writer_tx, mut ev_writer_rx) = mpsc::channel(128);

    // send one end of the communication channels to the readers/writer
    bind_udev_inputs(&configuration.devices, ev_reader_init_tx, ev_writer_tx).await?;
    let mut ev_reader_tx = ev_reader_init_rx.await?;




    // initial evaluation pass on global scope
    // {
    //     let execution_message_tx = execution_message_tx.clone();
    //     let ev_reader_tx = ev_reader_tx.clone();
    //     task::spawn(async move {
    //         script::evaluate_script(script_ast, execution_message_tx, ev_reader_tx, window_cycle_token).await;
    //     });
    // }
    //
    // // main processing loop
    // loop {
    //     tokio::select! {
    //         Some(window) = window_ev_rx.recv() => {
    //             state.active_window = Some(window);
    //             window_cycle_token = window_cycle_token + 1;
    //             event_handlers::handle_active_window_change(&mut ev_reader_tx,
    //                 &mut execution_message_tx, window_cycle_token, &mut window_change_handlers);
    //         }
    //         Some(ev) = ev_writer_rx.recv() => {
    //             event_handlers::handle_stdin_ev(
    //                 &mut state, ev,
    //                 &mut mappings,
    //                 &mut ev_reader_tx,
    //                 &mut execution_message_tx,
    //                 window_cycle_token,
    //                 &configuration,
    //             ).await.unwrap();
    //         }
    //         Some(msg) = message_rx.recv() => {
    //             event_handlers::handle_execution_message(&mut stdout, window_cycle_token, msg, &mut state,
    //                 &mut mappings, &mut window_change_handlers).await;
    //         }
    //     }
    // }
}
