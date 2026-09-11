use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_unused_interface_in_namespace2() {
    let content = r#"// @noUnusedLocals: true
namespace greeter {
    [| export interface interface2 {
    }
    interface interface1 {
    } |]
}"#;
    let mut s = Session::new_for_test("codeFixUnusedInterfaceInNamespace2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `export interface interface2 {
}
