use crate::*;
use crate::tests::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hjkl_arrow_keys_test() -> Result<()> {
    let mut params = ScriptTestingParameters::default();
    params.script_path = "examples/hjkl-arrow-keys.m2";

    let mut api = test_script(params).await?;
    api.event_delay = Some(200);

    api.write_action(KeyAction::new(*KEY_LEFT_ALT, 1)).await?;
    api.write_action(KeyAction::new(*KEY_H, 1)).await?;
    api.write_action(KeyAction::new(*KEY_H, 0)).await?;
    api.write_action(KeyAction::new(*KEY_LEFT_ALT, 0)).await?;
    sleep(200);

    let output_ev = api.collect_output_ev().await;

    assert_eq!(output_ev, vec![
        KeyAction::new(*KEY_LEFT_ALT, 1).to_input_ev(),
        KeyAction::new(*KEY_LEFT_ALT, 0).to_input_ev(),
        KeyAction::new(*KEY_LEFT, 1).to_input_ev(),
        SYN_REPORT.clone(),
        KeyAction::new(*KEY_LEFT, 0).to_input_ev(),
        SYN_REPORT.clone(),
        KeyAction::new(*KEY_LEFT_ALT, 1).to_input_ev(),
        KeyAction::new(*KEY_LEFT_ALT, 0).to_input_ev(),
    ]);

    api.stop().await;

    Ok(())
}