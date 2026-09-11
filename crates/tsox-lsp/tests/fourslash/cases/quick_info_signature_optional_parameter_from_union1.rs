use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_signature_optional_parameter_from_union1() {
    let content = r#"// @strict: false
declare const optionals:
  | ((a?: { a: true }) => unknown)
  | ((b?: { b: true }) => unknown);

/**/optionals();"#;
    let mut s = Session::new_for_test("quickInfoSignatureOptionalParameterFromUnion1", content);
    fourslash::verify_quick_info_at(&mut s, "", "const optionals: (arg0?: {\n    a: true;\n} & {\n    b: true;\n}) => unknown", "");
}
