use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_expected_comma03() {
    let content = r#"class C {
    const example = [|{ one: 1 one }|]
}"#;
    let mut s = Session::new_for_test("codeFixExpectedComma03", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixExpectedComma")
}
