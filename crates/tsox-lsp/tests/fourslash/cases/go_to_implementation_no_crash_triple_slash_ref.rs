use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_no_crash_triple_slash_ref() {
    let content = r#"// @Filename: /node_modules/@types/mymod/index.d.ts
export declare function foo(): void;
// @Filename: /main.d.ts
/// <reference types="/*m*/mymod" />"#;
    let _s = Session::new_for_test("goToImplementationNoCrashTripleSlashRef", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "m")
}
