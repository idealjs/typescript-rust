use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_deprecated_suggestion3() {
    let content = r#"interface RequestOptions {
    /** @deprecated use signal instead */
    timeout?: number;
}
declare function request(url: string, opts: RequestOptions): void;

request("/api", { [|timeout|]: 5000 });
declare const opts: RequestOptions;
opts.[|timeout|];"#;
    let _s = Session::new_for_test("jsdocDeprecated_suggestion3", content);
    // TODO: f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
