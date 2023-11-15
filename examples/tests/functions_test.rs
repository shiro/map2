// use crate::*;
// use crate::tests::*;
// use indoc::indoc;

// #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
// async fn functions_test() -> Result<()> {
//     let mut params = ScriptTestingParameters::default();
//     params.script_path = "examples/functions.m2";
//
//     let mut api = test_script(params).await.unwrap();
//     sleep(200);
//
//     let output = api.collect_stdout().await;
//
//     let expected = indoc! {"
//     hello world
//     hello from my_function
//     1 + 2 = 3
//     "};
//     assert_eq!(&*output, expected);
//
//     api.stop().await;
//
//     Ok(())
// }