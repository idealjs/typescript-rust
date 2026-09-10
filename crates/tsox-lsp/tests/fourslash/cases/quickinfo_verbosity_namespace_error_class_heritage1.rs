use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_namespace_error_class_heritage1() {
    let content = r#"
namespace NS/*1*/ {
    export class Derived extends NonExistentClass {
        derivedField: number;
    }
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNamespaceErrorClassHeritage1", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
