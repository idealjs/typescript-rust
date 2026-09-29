use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_deprecated_suggestion10() {
    let content = r#"// @filename: foo.ts
export namespace foo {
    /** @deprecated */
    export const bar = 1;
    [|bar|];
}
foo.[|bar|];
foo[[|"bar"|]];"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion10", content);
    fourslash::go_to_file(&mut s, "foo.ts");
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
