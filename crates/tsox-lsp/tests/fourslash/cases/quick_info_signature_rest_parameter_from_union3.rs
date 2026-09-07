use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_signature_rest_parameter_from_union3() {
    let content = r#"declare const fn:
  | ((a: { x: number }, b: { x: number }) => number)
  | ((...a: { y: number }[]) => number);

/**/fn();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "const fn: (a: {\n    x: number;\n} & {\n    y: number;\n}, b: {\n    x: 
}
