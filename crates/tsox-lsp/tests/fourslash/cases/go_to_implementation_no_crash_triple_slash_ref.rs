use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_no_crash_triple_slash_ref() {
    let content = r#"// @Filename: /node_modules/@types/mymod/index.d.ts
export declare function foo(): void;
// @Filename: /main.d.ts
/// <reference types="/*m*/mymod" />"#;
    let mut s = Session::new_for_test("goToImplementationNoCrashTripleSlashRef", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "m")
}
