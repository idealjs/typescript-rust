use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_weight1() {
    let content = r#"let ABCDEFGHIJKLMNOPQR = 1;
let abcdefghijklmnopqrs = 1;
[|abcdefghijklmnopqr|]"#;
    let _s = Session::new_for_test("codeFixSpellingCaseWeight1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `abcdefghijklmnopqrs`, false, 0, 0)
}
