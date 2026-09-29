use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_namespace_anonymous_class_heritage1() {
    let content = r#"
namespace NS/*1*/ {
    export class Derived extends class {
        baseField: string;
    } {
        derivedField: number;
    }
}
"#;
    let _s = Session::new_for_test("quickinfoVerbosityNamespaceAnonymousClassHeritage1", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
