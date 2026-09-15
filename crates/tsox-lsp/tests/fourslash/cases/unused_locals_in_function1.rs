use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_locals_in_function1() {
    let content = r#"// @noUnusedLocals: true
 [| function greeter() {
    var x = 0;
} |]"#;
    let _s = Session::new_for_test("unusedLocalsInFunction1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `
}
