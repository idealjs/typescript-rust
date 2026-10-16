use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_sensitive1() {
    let content = r#"export let Console = 1;
export let console = 1;
[|conole|] = 1;"#;
    let _s = Session::new_for_test("codeFixSpellingCaseSensitive1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `console`, false, 0, 0)
}
