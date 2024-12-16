#![feature(proc_macro_span)]

use proc_macro::TokenStream;

use parse::Parser;
use punctuated::*;
use syn::*;

#[proc_macro]
pub fn include_python(_item: TokenStream) -> TokenStream {
    let span = proc_macro::Span::call_site();
    let source = span.source_file();

    // TODO editor can't resolve the filepath, return a result instead
    let py_filename =
        format!("{}.py", source.path().file_stem().map(|v| v.to_string_lossy().to_string()).unwrap_or("".to_string()));

    format!("PyModule::from_code(py, pyo3::ffi::c_str!(include_str!(\"../{py_filename}\")), pyo3::ffi::c_str!(\"\"), pyo3::ffi::c_str!(\"\"))?")
        .parse()
        .unwrap()
}

#[proc_macro]
pub fn io_test(args: TokenStream) -> TokenStream {
    let parser = Punctuated::<LitStr, Token![,]>::parse_separated_nonempty;
    let mut args: Vec<String> = parser.parse(args).unwrap().iter().map(|v| v.value()).collect();

    let name = args.remove(0);
    let input = args.remove(0);
    let output = args.remove(0);

    format!(
        r#"
#[test_main]
async fn {name}() -> PyResult<()> {{
    Python::with_gil(|py| -> PyResult<()> {{
        let m = &include_python!();

        if !"{input}".is_empty() {{
            reader_send_all(py, m, "reader", &keys("{input}"));
        }}

        py.allow_threads(|| {{
            thread::sleep(Duration::from_millis(25));
        }});

        assert_eq_events!(writer_read_all(py, m, "writer"), keys("{output}"));

        Ok(())
    }})?;
    Ok(())
}}
"#
    )
    .parse()
    .unwrap()
}

enum IOTestAction {
    Input(String),
    Sleep(u32),
    Output(String),
    Global { name: String, value: String },
}
// impl syn::parse::Parse for IOTestAction {
//     fn parse(input: syn::parse::ParseStream<'_>) -> parse::Result<Self> {
//         let input = input.parse::<LitStr>().unwrap();
//         let input = input.value();
//
//         if let Ok(duration) = input.parse::<u32>() {
//             return Ok(IOTestAction::Sleep(duration));
//         };
//
//         return Ok(IOTestAction::Input(input));
//     }
// }

impl syn::parse::Parse for IOTestAction {
    fn parse(input: syn::parse::ParseStream<'_>) -> parse::Result<Self> {
        // let action = match input.parse::<Ident>()?.to_string().as_ref() {
        //     "input" => IOTestAction::Input(input.parse::<LitStr>()?.value()),
        //     "sleep" => IOTestAction::Sleep(input.parse::<LitInt>()?.base10_parse()?),
        //     "output" => IOTestAction::Output(input.parse::<LitStr>()?.value()),
        //     _ => panic!("unexpected token"),
        // };

        let input = input.parse::<LitStr>()?.value();
        let (ttype, value) = input.split_once(" ").expect("unexpected input");

        let action = match ttype {
            "input" => IOTestAction::Input(value.to_string()),
            "sleep" => IOTestAction::Sleep(value.parse().expect("failed to parse integer")),
            "output" => IOTestAction::Output(value.to_string()),
            "global" => {
                let (name, value) = value.split_once(" ").expect("expected name, value pair");
                let mut value = value.to_string();

                if value.parse::<i32>().is_err() {
                    value = format!("\"{value}\"");
                }

                IOTestAction::Global { name: name.to_string(), value: value.to_string() }
            }
            _ => panic!("unexpected action type"),
        };

        // "input" => IOTestAction::Input(input.parse::<LitStr>()?.value()),
        // "sleep" => IOTestAction::Sleep(input.parse::<LitInt>()?.base10_parse()?),
        // "output" => IOTestAction::Output(input.parse::<LitStr>()?.value()),
        // _ => panic!("unexpected token"),
        // };

        // match input.parse::<Lit>().expect("expected a literal") {
        //     Lit::Str(v) => Ok(IOTestAction::Input(v.value())),
        //     Lit::Int(v) => Ok(IOTestAction::Sleep(v.base10_parse()?)),
        //     _ => panic!("expected string or integer"),
        // }

        // let variant = input.parse::<Variant>().expect("expected an enum variant");
        // println!("{:?}", (input));
        // Ok(IOTestAction::Sleep(22))
        Ok(action)
    }
}

struct Foo {
    name: String,
    actions: Vec<IOTestAction>,
}

impl syn::parse::Parse for Foo {
    fn parse(input: syn::parse::ParseStream<'_>) -> parse::Result<Self> {
        let name = input.parse::<LitStr>().expect("expected test name").value();
        input.parse::<Token![,]>().expect("expected comma");

        let actions: Vec<_> =
            input.parse_terminated(|stream| stream.parse::<IOTestAction>(), Token![,])?.into_iter().collect();

        Ok(Self { name, actions })
    }
}

#[proc_macro]
pub fn io_test2(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Foo);
    let name = input.name;
    // let name = parse::<LitStr>(args).expect("expected test name").value();
    // parse::<Token![,]>(args).expect("expected comma");

    // let parser = Punctuated::<IOTestAction, Token![,]>::parse_separated_nonempty;
    // let args = parser.parse(args).expect("failed to parse");
    // let mut args: Vec<_> = args.iter().collect();

    // let name = "foob".to_string();
    // let name = match args.remove(0) {
    //     IOTestAction::Input(v) => v,
    //     _ => unreachable!(),
    // };
    // let output = match args.pop().unwrap() {
    //     IOTestAction::Input(v) => v,
    //     _ => unreachable!(),
    // };

    // input.actions.push(IOTestAction::Output("".to_string()));

    let test_code = input.actions.iter().fold("".to_string(), |acc, action| match action {
        IOTestAction::Input(v) => {
            let code: String = format!(r#"reader_send_all(py, m, "reader", &keys("{v}"));"#).parse().unwrap();
            acc + &code
        }
        IOTestAction::Sleep(v) => {
            let code: String = format!(
                r#"
                    py.allow_threads(|| {{
                        thread::sleep(Duration::from_millis({v}));
                    }});
            "#
            )
            .parse()
            .unwrap();
            acc + &code
        }
        IOTestAction::Output(v) => {
            let code: String = format!(
                r#"
                py.allow_threads(|| {{
                    thread::sleep(Duration::from_millis(10));
                }});
                assert_eq_events!(writer_read_all(py, m, "writer"), keys("{v}"));
            "#
            )
            .parse()
            .unwrap();
            acc + &code
        }
        IOTestAction::Global { name, value } => {
            let code: String = format!(
                r#"
                    let value = m.getattr("{name}").unwrap().extract::<i32>().unwrap();
                    assert_eq!(value, {value});
                "#
            )
            .parse()
            .unwrap();
            acc + &code
        }
    });

    format!(
        r#"
#[test_main]
async fn {name}() -> PyResult<()> {{
    Python::with_gil(|py| -> PyResult<()> {{
        let m = &include_python!();

        {test_code}

        Ok(())
    }})?;
    Ok(())
}}
"#
    )
    .parse()
    .unwrap()
}

// #[proc_macro_attribute]
// pub fn simple_io_test(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let input = syn::parse_macro_input!(item as syn::ItemFn);
//
//     let sig = &input.sig;
//     let name = &input.sig.ident;
//     let body = &input.block;
//     let vis = &input.vis;
//     // println!("attr: \"{attr}\"");
//     // println!("item: \"{item}\"");
//     // item
//     format!(
//         r#"
// #[test_main]
// async fn a_to_b() -> PyResult<()> {{
//     Python::with_gil(|py| -> PyResult<()> {{
//         let m = &include_python!();
//
//         reader_send_all(py, m, "reader", &keys("a"));
//
//         py.allow_threads(|| {{
//             thread::sleep(Duration::from_millis(25));
//         }});
//
//         assert_eq!(writer_read_all(py, m, "writer"), keys("b"),);
//
//         Ok(())
//     }})?;
//     Ok(())
// }}
// "#
//     )
//     .parse()
//     .unwrap()
// }
