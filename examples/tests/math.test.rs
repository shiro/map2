#[cfg(test)]
mod test {
    use crate::*;
    use crate::messaging::ExecutionMessage;
    use indoc::indoc;

    #[tokio::test]
    async fn math_test() -> Result<()> {
        let mut script_file = fs::File::open("examples/math.m2")?;

        let script_ast = script::parse_script(&mut script_file);

        let (execution_message_tx, mut execution_message_rx) = mpsc::channel(128);
        let (ev_reader_tx, _) = mpsc::channel(128);

        script::evaluate_script(script_ast, execution_message_tx, ev_reader_tx, 0).await;

        let mut output = String::new();
        while let Some(msg) = execution_message_rx.recv().await {
            match msg {
                ExecutionMessage::Write(part) => { output.push_str(&*part) }
                _ => {}
            }
        }

        let expected = indoc! {"
        sum of 1 and 2 is: 3
        1 minus 2 is: -1
        2 times 4 is: 8
        4 divided by 2 is: 2
        result of complicated calculation: 3.8
    "};
        assert_eq!(&*output, expected);

        Ok(())
    }
}