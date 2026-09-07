use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn unused_variable_in_class5() {
    let content = r#"// @noUnusedLocals: true
// @target: esnext
declare class greeter {
    #private;
    private name;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
