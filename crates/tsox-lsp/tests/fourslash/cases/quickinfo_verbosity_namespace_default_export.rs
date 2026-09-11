use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_verbosity_namespace_default_export() {
    let content = r#"
declare namespace ns/*1*/ {
    interface Shape {
        sides: number;
    }
    const circle: Shape;
    export default circle;
    export { Shape };
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNamespaceDefaultExport", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
