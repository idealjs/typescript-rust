use tsox_lsp::fourslash::{self, Session};


#[test]
fn unused_locals_in_function1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
 [| function greeter() {
    var x = 0;
} |]"#;
    let mut s = Session::new_for_test("unusedLocalsInFunction1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `
}
