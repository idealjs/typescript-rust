use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_unused_interface_in_namespace2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
namespace greeter {
    [| export interface interface2 {
    }
    interface interface1 {
    } |]
}"#;
    let mut s = Session::new_for_test("codeFixUnusedInterfaceInNamespace2", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `export interface interface2 {
}
