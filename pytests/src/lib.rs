#![feature(proc_macro_span)]

#[proc_macro]
pub fn include_python(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let span = proc_macro::Span::call_site();
    let source = span.source_file();

    // TODO editor can't resolve the filepath, return a result instead
    let py_filename =
        format!("{}.py", source.path().file_stem().map(|v| v.to_string_lossy().to_string()).unwrap_or("".to_string()));

    format!("PyModule::from_code_bound(py, include_str!(\"../{py_filename}\"), \"\", \"\")?").parse().unwrap()
}
