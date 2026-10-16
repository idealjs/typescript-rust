use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_namespace_interface_heritage_crash() {
    let content = r#"
declare namespace NS/*1*/ {
    interface Config extends Record<string, any> {}
}
"#;
    let _s = Session::new_for_test("quickinfoVerbosityNamespaceInterfaceHeritageCrash", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
