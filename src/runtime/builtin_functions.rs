use evdev_rs::enums::int_to_ev_key;
use tokio::process::Command;

use crate::*;
use crate::messaging::ExecutionMessage;
use crate::parsing::parser::{parse_key_action_with_mods, parse_key_sequence};

pub async fn throw_error<'a>(err: anyhow::Error, exit_code: i32, amb: &mut Ambient<'a>) -> ValueType {
    amb.message_tx.borrow_mut().as_ref().unwrap()
        .send(ExecutionMessage::FatalError(err, exit_code))
        .await.unwrap();
    return ValueType::Void;
}

pub async fn evaluate_builtin<'a>(name: &String, args: &Vec<Expr>, var_map: &GuardedVarMap, amb: &mut Ambient<'_>) -> Result<ValueType> {
    let mut parsed_args = vec![];
    for expr in args {
        let arg = eval_expr(expr, var_map, amb).await;
        parsed_args.push(arg);
    }

    match &**name {
        "exit" => {
            let arg = args.get(0);
            let val = match arg {
                Some(arg) => eval_expr(arg, var_map, amb).await,
                _ => ValueType::Number(0.0),
            };

            let exit_code = match val {
                ValueType::Number(exit_code) => exit_code as i32,
                _ => return Err(anyhow!("the first parameter to 'exit' must be a number")),
            };

            amb.message_tx.as_ref().unwrap().send(ExecutionMessage::Exit(exit_code)).await.unwrap();
        }
        "send" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::String(val) => val,
                _ => return Err(anyhow!("invalid parameter passed to function 'send'")),
            };

            let actions = parse_key_sequence(&*val).unwrap();

            for action in actions {
                amb.ev_writer_tx.send(action.to_input_ev()).await.unwrap();
                amb.ev_writer_tx.send(SYN_REPORT.clone()).await.unwrap();
            }
        }

        "active_window_class" => {
            let (tx, mut rx) = mpsc::channel(1);
            amb.message_tx.as_ref().unwrap().send(ExecutionMessage::GetFocusedWindowInfo(tx)).await.unwrap();
            if let Some(active_window) = rx.recv().await.unwrap() {
                return Ok(ValueType::String(active_window.class));
            }
        }
        "on_window_change" => {
            if args.len() != 1 {
                return Err(anyhow!("function takes 1 argument"));
            }

            let inner_block;
            let inner_var_map;
            if let ValueType::Lambda(_, _block, _var_map) = eval_expr(args.get(0).unwrap(), var_map, amb).await {
                inner_block = _block;
                inner_var_map = _var_map;
            } else {
                return Err(anyhow!("type mismatch, function takes lambda argument"));
            }

            amb.message_tx.as_ref().unwrap().send(ExecutionMessage::RegisterWindowChangeCallback(inner_block, inner_var_map)).await.unwrap();
        }
        "sleep" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            match val {
                ValueType::Number(millis) => tokio::time::sleep(time::Duration::from_millis(millis as u64)).await,
                _ => return Err(anyhow!("sleep expects a number argument")),
            }
        }
        "print" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = format!("{}\n", val);

            amb.message_tx.borrow_mut().as_ref().unwrap()
                .send(ExecutionMessage::Write(val)).await
                .unwrap();
        }
        "number_to_key" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::Number(val) => val,
                _ => return Err(anyhow!("only numbers can be converted to keys")),
            };
            let val = val as u32;

            let key = int_to_ev_key(val).expect(&*format!("key for scan code '{}' not found", val));

            return Ok(ValueType::String(format!("{{{}}}", EventCode::EV_KEY(key).to_string())));
        }
        "number_to_char" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::Number(val) => val,
                _ => return Err(anyhow!("only numbers can be converted to chars")),
            };

            let val = val as u8 as char;
            return Ok(ValueType::String(format!("{}", val)));
        }
        "char_to_number" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::String(val) => val,
                _ => return Err(anyhow!("only chars can be converted to chars")),
            };
            if val.len() != 1 { panic!("string needs to contain exactly 1 character") }

            let first_ch = val.chars().next().unwrap();
            let val = first_ch as u8 as f64;
            return Ok(ValueType::Number(val));
        }
        "map_key" => {
            let val = (
                eval_expr(args.get(0).unwrap(), var_map, amb).await,
                eval_expr(args.get(1).unwrap(), var_map, amb).await,
            );
            let (from, to) = match val {
                (ValueType::String(from), ValueType::Lambda(_, to, var_map)) => (from, (to, var_map)),
                _ => return Err(anyhow!("invalid arguments passed to 'map_key'")),
            };

            let mappings = match parse_key_action_with_mods(&*from, to.0).unwrap() {
                Expr::KeyMapping(v) => v,
                _ => unreachable!(),
            };

            for mapping in mappings {
                let mapping = mapping.clone();

                amb.message_tx.borrow_mut().as_ref().unwrap()
                    .send(ExecutionMessage::AddMapping(amb.window_cycle_token, mapping.from, mapping.to, to.1.clone())).await
                    .unwrap();
            }
        }
        "execute" => {
            if parsed_args.len() < 1 { return Err(anyhow!("argument error: function 'execute' expected at least 1 argument")); }

            let parsed_args = parsed_args.iter().map(|val| match val {
                ValueType::String(v) => Ok(v.to_string()),
                ValueType::Number(v) => Ok(v.to_string()),
                v => return Err(anyhow!("unexpected argument passed to 'execute': '{}'", v)),
            }).collect::<Result<Vec<String>>>()?;

            // crate a system command
            let mut cmd = Command::new(parsed_args.get(0).unwrap());

            // append arguments to command
            for arg in parsed_args.iter().skip(1) { cmd.arg(arg); }

            let child_process = cmd.output().await
                .map_err(|err| anyhow!("failed to spawn child process: {}", err))?;

            // trim trailing newline
            let mut output = String::from_utf8_lossy(&child_process.stdout).to_string();
            if output.ends_with('\n') {
                output.pop();
                if output.ends_with('\r') {
                    output.pop();
                }
            }

            return Ok(ValueType::String(output.to_string()));
        }
        name => {
            let (lambda_params, lambda_block, lambda_var_map) = match eval_expr(&Expr::Name(name.to_string()), var_map, amb).await {
                ValueType::Lambda(params, block, var_map) => (params, block, var_map),
                ValueType::Void => return Err(anyhow!("function '{}' not found in this scope", name)),
                _ => return Err(anyhow!("variable '{}' is not a lambda function", name)),
            };

            // we need to clone the lambda's var_map since each lambda execution needs to not affect the next one
            // TODO make GuardedVarMap a proper struct and implement a proper deep clone method
            let mut lambda_var_map = GuardedVarMap::new(Mutex::new(VarMap::new(
                lambda_var_map.lock().unwrap().parent.clone()
            )));

            for (idx, param) in lambda_params.iter().enumerate() {
                let val = match args.get(idx) {
                    Some(expr) => eval_expr(expr, var_map, amb).await,
                    None => ValueType::Void,
                };

                eval_expr(&Expr::Init(param.clone(), Box::new(Expr::Value(val))), &lambda_var_map, amb).await;
            }

            let ret = eval_block(&lambda_block, &mut lambda_var_map, amb).await;
            match ret {
                BlockRet::Return(ret) => return Ok(ret),
                BlockRet::Continue => return Err(anyhow!("function cannot return a continue statement")),
                BlockRet::None => {}
            }
        }
    };

    Ok(ValueType::Void)
}
