use crate::*;
use crate::tests::*;
use indoc::indoc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn math_test() -> Result<()> {
    let mut params = ScriptTestingParameters::default();
    params.script_path = "examples/math.m2";

    let mut api = test_script(params).await.unwrap();
    sleep(200);

    let output = api.collect_stdout().await;

    let expected = indoc! {"
    sum of 1 and 2 is: 3
    1 minus 2 is: -1
    2 times 4 is: 8
    4 divided by 2 is: 2
    result of complicated calculation: 3.8
    "};
    assert_eq!(&*output, expected);

    api.stop().await;

    Ok(())
}