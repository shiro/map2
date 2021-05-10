use crate::*;
use crate::messaging::ExecutionMessage;
use evdev_rs::enums::int_to_ev_key;
use crate::parsing::parser::{parse_key_action_with_mods, parse_key_sequence};

pub async fn evaluate_builtin<'a>(name: &String, args: &Vec<Expr>, var_map: &GuardedVarMap, amb: &mut Ambient<'_>) -> ValueType {
    match &**name {
        "exit" => {
            let arg = args.get(0);
            let val = match arg {
                Some(arg) => eval_expr(arg, var_map, amb).await,
                _ => ValueType::Number(0.0),
            };

            let exit_code = match val {
                ValueType::Number(exit_code) => exit_code as i32,
                _ => panic!("the first parameter to 'exit' must be a number"),
            };

            amb.message_tx.as_ref().unwrap().send(ExecutionMessage::Exit(exit_code)).await.unwrap();
            ValueType::Void
        }
        "send" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::String(val) => val,
                _ => panic!("invalid parameter passed to function 'send'"),
            };

            let actions = parse_key_sequence(&*val).unwrap();

            for action in actions {
                amb.ev_writer_tx.send(action.to_input_ev()).await.unwrap();
                amb.ev_writer_tx.send(SYN_REPORT.clone()).await.unwrap();
            }

            ValueType::Void
        }

        "active_window_class" => {
            let (tx, mut rx) = mpsc::channel(1);
            amb.message_tx.as_ref().unwrap().send(ExecutionMessage::GetFocusedWindowInfo(tx)).await.unwrap();
            if let Some(active_window) = rx.recv().await.unwrap() {
                return ValueType::String(active_window.class);
            }
            ValueType::Void
        }
        "on_window_change" => {
            if args.len() != 1 { panic!("function takes 1 argument") }

            let inner_block;
            let inner_var_map;
            if let ValueType::Lambda(_, _block, _var_map) = eval_expr(args.get(0).unwrap(), var_map, amb).await {
                inner_block = _block;
                inner_var_map = _var_map;
            } else {
                panic!("type mismatch, function takes lambda argument");
            }

            amb.message_tx.as_ref().unwrap().send(ExecutionMessage::RegisterWindowChangeCallback(inner_block, inner_var_map)).await.unwrap();
            ValueType::Void
        }
        "sleep" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            match val {
                ValueType::Number(millis) => tokio::time::sleep(time::Duration::from_millis(millis as u64)).await,
                _ => panic!("sleep expects a number argument"),
            }

            ValueType::Void
        }
        "print" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            println!("{}", val);
            ValueType::Void
        }
        "number_to_key" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::Number(val) => val,
                _ => panic!("only numbers can be converted to keys"),
            };
            let val = val as u32;

            let key = int_to_ev_key(val).expect(&*format!("key for scan code '{}' not found", val));

            ValueType::String(format!("{{{}}}", EventCode::EV_KEY(key).to_string()))
        }
        "number_to_char" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::Number(val) => val,
                _ => panic!("only numbers can be converted to chars"),
            };

            let val = val as u8 as char;
            ValueType::String(format!("{}", val))
        }
        "char_to_number" => {
            let val = eval_expr(args.get(0).unwrap(), var_map, amb).await;
            let val = match val {
                ValueType::String(val) => val,
                _ => panic!("only chars can be converted to chars"),
            };
            if val.len() != 1 { panic!("string needs to contain exactly 1 character") }

            let first_ch = val.chars().next().unwrap();
            let val = first_ch as u8 as f64;
            ValueType::Number(val)
        }
        "map_key" => {
            let val = (
                eval_expr(args.get(0).unwrap(), var_map, amb).await,
                eval_expr(args.get(1).unwrap(), var_map, amb).await,
            );
            let (from, to) = match val {
                (ValueType::String(from), ValueType::Lambda(_, to, var_map)) => (from, (to, var_map)),
                _ => panic!("invalid arguments passed to 'map_key'"),
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

            ValueType::Void
        }
        name => {
            let (lambda_params, lambda_block, lambda_var_map) = match eval_expr(&Expr::Name(name.to_string()), var_map, amb).await {
                ValueType::Lambda(params, block, var_map) => (params, block, var_map),
                ValueType::Void => panic!("function '{}' not found in this scope", name),
                _ => panic!("variable '{}' is not a lambda function", name),
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
                BlockRet::None => ValueType::Void,
                BlockRet::Return(ret) => ret,
                BlockRet::Continue => panic!("function cannot return a continue statement"),
            }
        }
    }
}
