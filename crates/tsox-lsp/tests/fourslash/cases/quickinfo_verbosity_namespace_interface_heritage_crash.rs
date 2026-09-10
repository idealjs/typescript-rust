use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_namespace_interface_heritage_crash() {
    let content = r#"
declare namespace NS/*1*/ {
    interface Config extends Record<string, any> {}
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNamespaceInterfaceHeritageCrash", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
