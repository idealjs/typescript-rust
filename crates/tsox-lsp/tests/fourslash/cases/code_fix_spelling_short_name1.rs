use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_short_name1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"export let ab = 1;
[|aB|] = 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `ab`, false, 0, 0)
}
