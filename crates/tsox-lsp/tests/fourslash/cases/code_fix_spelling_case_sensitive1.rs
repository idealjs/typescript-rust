use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_sensitive1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"export let Console = 1;
export let console = 1;
[|conole|] = 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `console`, false, 0, 0)
}
