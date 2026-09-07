use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_signature_rest_parameter_from_union1() {
    let content = r#"declare const rest:
  | ((v: { a: true }, ...rest: string[]) => unknown)
  | ((v: { b: true }) => unknown);

/**/rest({ a: true, b: true }, "foo", "bar");"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "",
        "const rest: (v: {\n    a: true;\n} & {\n    b: true;\n}, ...rest: string[]) => unknown",
        "",
    );
}
