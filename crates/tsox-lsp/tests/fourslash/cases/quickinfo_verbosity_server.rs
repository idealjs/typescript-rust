use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_server() {
    let content = r#"// @lib: es5
type FooType = string | number
const foo/*a*/: FooType = 1"#;
    let _s = Session::new_for_test("quickinfoVerbosityServer", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"a": {0, 1}})
}
