use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
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
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "foo.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
