use tsox_lsp::fourslash::{self, Session};

#[test]
fn unused_variable_in_class5() {
    let content = r#"// @noUnusedLocals: true
// @target: esnext
declare class greeter {
    #private;
    private name;
}"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
}
