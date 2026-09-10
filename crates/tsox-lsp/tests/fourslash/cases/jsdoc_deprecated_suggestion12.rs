use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion12() {
    let content = r#"// @filename: foo.ts
/**
 * @deprecated
 */
function foo() {};
function bar(fn: () => void) {
    fn();
}
bar([|foo|]);"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion12", content);
    fourslash::go_to_file(&mut s, "foo.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
