use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_spelling_short_name2() {
    let content = r#"export let ab = 1;
abc;"#;
    let mut s = Session::new_for_test("codeFixSpellingShortName2", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
