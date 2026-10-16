use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_localsin_constructor_fs1() {
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters:true
class greeter {
    [| constructor() {
        var unused = 20;
    } |]
}"#;
    let _s = Session::new_for_test("unusedLocalsinConstructorFS1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `constructor() {
}
