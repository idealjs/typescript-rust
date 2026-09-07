use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_signature_optional_parameter_from_union1() {
    let content = r#"// @strict: false
declare const optionals:
  | ((a?: { a: true }) => unknown)
  | ((b?: { b: true }) => unknown);

/**/optionals();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "const optionals: (arg0?: {\n    a: true;\n} & {\n    b: true;\n}) => unk
}
