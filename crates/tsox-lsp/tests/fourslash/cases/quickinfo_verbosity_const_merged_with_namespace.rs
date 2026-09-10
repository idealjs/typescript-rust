use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_const_merged_with_namespace() {
    let content = r#"
declare function create/*1*/(x: string): number;
declare namespace create/*2*/ {
    var version: string;
    function reset(): void;
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityConstMergedWithNamespace", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{
}
