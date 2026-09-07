use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion22() {
    let content = r#"// @filename: /a.ts
const foo: {
    /**
	 * @deprecated
	 */
	(a: string, b: string): string;
	(a: string, b: number): string;
} = (a: string, b: string | number) => a + b;

[|foo|](1, 1);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}
