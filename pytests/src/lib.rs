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
