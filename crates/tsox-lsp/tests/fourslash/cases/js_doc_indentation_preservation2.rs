use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoIs"]
#[test]
fn js_doc_indentation_preservation2() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/**
	Does some stuff.
	    Second line.
		Third line.
*/
function foo/**/(){}"#;
    let mut s = Session::new_for_test("jsDocIndentationPreservation2", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "function foo(): void", "Does some stuff.\n    Second line.\n\tThird line.")
}
