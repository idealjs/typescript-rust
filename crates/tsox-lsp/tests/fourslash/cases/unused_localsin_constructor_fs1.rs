use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_localsin_constructor_fs1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters:true
class greeter {
    [| constructor() {
        var unused = 20;
    } |]
}"#;
    let mut s = Session::new_for_test("unusedLocalsinConstructorFS1", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `constructor() {
}
