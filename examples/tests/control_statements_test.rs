use crate::*;
use crate::tests::*;
use indoc::indoc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn control_statements_test() -> Result<()> {
    let mut params = ScriptTestingParameters::default();
    params.script_path = "examples/control-statements.m2";

    let mut api = test_script(params).await.unwrap();
    sleep(200);

    let output = api.collect_stdout().await;

    let expected = indoc! {"
    a is 3
    i is 0
    i is 1
    i is 2
    i is 4
    "};
    assert_eq!(&*output, expected);

    api.stop().await;

    Ok(())
}