use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("jsdocDeprecated_suggestion22", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
