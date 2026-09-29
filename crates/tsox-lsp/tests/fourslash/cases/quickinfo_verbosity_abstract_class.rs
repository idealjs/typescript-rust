use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_abstract_class() {
    let content = r#"
declare abstract class Shape/*1*/ {
    abstract area(): number;
    abstract perimeter(): number;
    toString(): string;
}
"#;
    let _s = Session::new_for_test("quickinfoVerbosityAbstractClass", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
