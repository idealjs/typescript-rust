use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_namespace_error_class_heritage1() {
    let content = r#"
namespace NS/*1*/ {
    export class Derived extends NonExistentClass {
        derivedField: number;
    }
}
"#;
    let _s = Session::new_for_test("quickinfoVerbosityNamespaceErrorClassHeritage1", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
