use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn js_doc_function_signatures6() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/**
 * @param {string} p1 - A string param
 * @param {string?} p2 - An optional param
 * @param {string} [p3] - Another optional param
 * @param {string} [p4="test"] - An optional param with a default value
 */
function f1(p1, p2, p3, p4){}
f1(/*1*/'foo', /*2*/'bar', /*3*/'baz', /*4*/'qux');"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures6", content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
