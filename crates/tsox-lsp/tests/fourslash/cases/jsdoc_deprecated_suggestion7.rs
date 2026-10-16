use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_deprecated_suggestion7() {
    let content = r#"enum Direction {
    Left = -1,
    Right = 1,
}
type T = Direction.Left
/** @deprecated */
const x = 1
type x = string
var y: x = 'hi'"#;
    let _s = Session::new_for_test("jsdocDeprecated_suggestion7", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
