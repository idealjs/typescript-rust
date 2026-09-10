use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_sensitive2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"export let console = 1;
export let Console = 1;
[|conole|] = 1;"#;
    let mut s = Session::new_for_test("codeFixSpellingCaseSensitive2", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `console`, false, 0, 0)
}
