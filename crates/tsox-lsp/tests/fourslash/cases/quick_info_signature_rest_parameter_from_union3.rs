use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_signature_rest_parameter_from_union3() {
    let content = r#"declare const fn:
  | ((a: { x: number }, b: { x: number }) => number)
  | ((...a: { y: number }[]) => number);

/**/fn();"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "",
        "const fn: (a: {\n    x: number;\n} & {\n    y: number;\n}, b: {\n    x: number;\n} & {\n    y: number;\n}, ...args: {\n    y: number;\n}[]) => number",
        "",
    );
}
