use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling4() {
    let content = r#"export declare const despite: { the: any };

[|dispite.the|]"#;
    let mut s = Session::new_for_test("codeFixSpelling4", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `despite.the`, false, 0, 0)
}
