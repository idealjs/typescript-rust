use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_symbols4() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @target: es6
// @Filename: file.js
const _sym = Symbol("_sym");
class MyClass {
    constructor() {
        // Dynamic assignment properties can't show up in navigation,
        // as they're not syntactic members
        // Additonally, late bound members are always filtered out, besides
        this[_sym] = "ok";
    }

    method() {
        this[_sym] = "yep";
        const x = this[_sym];
    }
}"#;
    let mut s = Session::new_for_test("navigationBarItemsSymbols4", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
