use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
