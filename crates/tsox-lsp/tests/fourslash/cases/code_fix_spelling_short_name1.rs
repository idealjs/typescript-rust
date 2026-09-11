use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_spelling_short_name1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"export let ab = 1;
[|aB|] = 1;"#;
    let mut s = Session::new_for_test("codeFixSpellingShortName1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `ab`, false, 0, 0)
}
