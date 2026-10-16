use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_weight2() {
    let content = r#"let ABCDEFGHI = 1;
let abcdefghij = 1;
[|abcdefghi|]"#;
    let _s = Session::new_for_test("codeFixSpellingCaseWeight2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `ABCDEFGHI`, false, 0, 0)
}
