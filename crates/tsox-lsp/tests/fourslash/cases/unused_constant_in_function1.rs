use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_constant_in_function1() {
    let content = r#"// @noUnusedLocals: true
[| function f1 () {
    const x: string = "x";
} |]"#;
    let mut s = Session::new_for_test("unusedConstantInFunction1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `function f1 () {
}
