use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_function_signatures12() {
    let content = r#"// @allowJs: true
// @Filename: jsDocFunctionSignatures.js
/**
 * @param {{
 *   stringProp: string,
 *   numProp: number,
 *   boolProp: boolean,
 *   anyProp: any,
 *   anotherAnyProp: any,
 *   functionProp: (arg0: string, arg1: any) => any
 * }} o
 */
function f1(o) {
    o/**/;
}"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures12", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "(parameter) o: {\n    stringProp: string;\n    numProp: number;\n    boolPro
}
