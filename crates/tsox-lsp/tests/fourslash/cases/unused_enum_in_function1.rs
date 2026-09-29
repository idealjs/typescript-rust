use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_enum_in_function1() {
    let content = r#"// @noUnusedLocals: true
[| function f1 () {
    enum Directions { Up, Down}
} |]"#;
    let _s = Session::new_for_test("unusedEnumInFunction1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `function f1 () {
}
