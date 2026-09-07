use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_signature_rest_parameter_from_union2() {
    let content = r#"// @strict: false
declare const rest:
  | ((a?: { a: true }, ...rest: string[]) => unknown)
  | ((b?: { b: true }) => unknown);

/**/rest({ a: true, b: true }, "foo", "bar");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "const rest: (arg0?: {\n    a: true;\n} & {\n    b: true;\n}, ...rest: st
}
