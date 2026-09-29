use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_localsin_constructor_fs2() {
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters: true
class greeter {
    [|constructor() {
        var unused = 20;
        var used = "dummy";
        used = used + "second part";
    }|]
}"#;
    let _s = Session::new_for_test("unusedLocalsinConstructorFS2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `
}
