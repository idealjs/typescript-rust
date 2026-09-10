use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_namespace_interface_heritage_intersection_crash() {
    let content = r#"
declare namespace NS/*1*/ {
    type Mixin = { a: string } & { b: number };
    interface Config extends Mixin {}
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNamespaceInterfaceHeritageIntersectionCrash", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
