use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_in_invalid_index_signature() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function method() { var /**/dictionary = <{ [index]: string; }>{}; }"#;
    let mut s = Session::new_for_test("quickInfoInInvalidIndexSignature", content);
    fourslash::verify_quick_info_at(&mut s, "", "(local var) dictionary: {\n    [x: number]: string;\n}", "");
}
