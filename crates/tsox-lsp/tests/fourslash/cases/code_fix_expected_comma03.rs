use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_expected_comma03() {
    let content = r#"class C {
    const example = [|{ one: 1 one }|]
}"#;
    let _s = Session::new_for_test("codeFixExpectedComma03", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixExpectedComma")
}
