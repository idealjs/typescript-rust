use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_abstract_class() {
    let content = r#"
declare abstract class Shape/*1*/ {
    abstract area(): number;
    abstract perimeter(): number;
    toString(): string;
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityAbstractClass", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
