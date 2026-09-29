use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_indentation_preservation3() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/**
    Does some stuff.
        Second line.
    	Third line.
*/
function foo/**/(){}"#;
    let mut s = Session::new_for_test("jsDocIndentationPreservation3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "function foo(): void", "Does some stuff.\n    Second line.\n\tThird line.")
}
