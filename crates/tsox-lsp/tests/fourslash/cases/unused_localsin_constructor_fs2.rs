use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_localsin_constructor_fs2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters: true
class greeter {
    [|constructor() {
        var unused = 20;
        var used = "dummy";
        used = used + "second part";
    }|]
}"#;
    let mut s = Session::new_for_test("unusedLocalsinConstructorFS2", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `
}
