#![feature(proc_macro_span)]

#[proc_macro]
pub fn include_python(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let span = proc_macro::Span::call_site();
    let source = span.source_file();

    let py_filename = format!("{}.py", source.path().file_stem().unwrap().to_string_lossy());

    format!("PyModule::from_code(py, include_str!(\"../{py_filename}\"), \"\", \"\")?").parse().unwrap()
}
