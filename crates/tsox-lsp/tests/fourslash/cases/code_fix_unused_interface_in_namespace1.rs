use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_unused_interface_in_namespace1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noUnusedLocals: true
 [| namespace greeter {
    interface interface1 {
    }
} |]"#;
    let mut s = Session::new_for_test("codeFixUnusedInterfaceInNamespace1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `
}
