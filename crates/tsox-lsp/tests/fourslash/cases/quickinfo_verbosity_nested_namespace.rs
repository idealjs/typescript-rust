use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_nested_namespace() {
    let content = r#"
declare namespace Outer/*1*/ {
    namespace Inner {
        const x: number;
        function f(): string;
    }
    const outerVal: boolean;
}
"#;
    let _s = Session::new_for_test("quickinfoVerbosityNestedNamespace", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
