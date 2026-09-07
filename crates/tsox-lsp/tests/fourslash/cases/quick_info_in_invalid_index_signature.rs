use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_in_invalid_index_signature() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function method() { var /**/dictionary = <{ [index]: string; }>{}; }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(local var) dictionary: {\n    [x: number]: string;\n}", "")
}
