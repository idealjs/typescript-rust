use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("jsdocDeprecated_suggestion19", content);
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
