use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion19() {
    let content = r#"interface I {
    x: number;
    y: number;
}
interface I {
    /** @deprecated  */
    x: number;
}
const foo: I = { [|x|]: 1, y: 1 };
foo.[|x|];"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion19", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
