use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_expected_comma03() {
    let content = r#"class C {
    const example = [|{ one: 1 one }|]
}"#;
    let mut s = Session::new_for_test("codeFixExpectedComma03", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixExpectedComma")
}
