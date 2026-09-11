use tsox_lsp::fourslash::{self, Session};


#[test]
fn side_effect_imports_suggestion1() {
    let content = r#"// @allowJs: true
// @noEmit: true
// @module: commonjs
// @noUncheckedSideEffectImports: true
// @filename: moduleA/a.js
import "b";
import "c";
// @filename: node_modules/b.ts
var a = 10;
// @filename: node_modules/c.js
exports.a = 10;
c = 10;"#;
    let mut s = Session::new_for_test("sideEffectImportsSuggestion1", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
