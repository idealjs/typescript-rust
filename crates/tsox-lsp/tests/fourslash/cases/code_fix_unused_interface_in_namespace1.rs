use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_unused_interface_in_namespace1() {
    let content = r#"// @noUnusedLocals: true
 [| namespace greeter {
    interface interface1 {
    }
} |]"#;
    let _s = Session::new_for_test("codeFixUnusedInterfaceInNamespace1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `
}
