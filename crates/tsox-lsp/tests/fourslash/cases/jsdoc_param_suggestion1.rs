use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_param_suggestion1() {
    let content = r#"// @Filename: a.ts
/**
 * @param options - whatever
 * @param options.zone - equally bad
 */
declare function bad(options: any): void

/**
 * @param {number} obtuse
 */
function worse(): void {
    arguments
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "a.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}
