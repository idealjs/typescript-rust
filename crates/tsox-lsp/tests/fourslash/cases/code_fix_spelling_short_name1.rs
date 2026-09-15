use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_short_name1() {
    let content = r#"export let ab = 1;
[|aB|] = 1;"#;
    let _s = Session::new_for_test("codeFixSpellingShortName1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `ab`, false, 0, 0)
}
