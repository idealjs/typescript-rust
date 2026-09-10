use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoIs"]
#[test]
fn js_doc_function_signatures11() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/**
 * @type {{ [name: string]: string; }} variables
 */
const vari/**/ables = {};"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures11", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "const variables: {\n    [name: string]: string;\n}", "")
}
