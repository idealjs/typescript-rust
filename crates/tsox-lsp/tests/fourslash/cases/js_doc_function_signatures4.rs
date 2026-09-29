use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_function_signatures4() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @param {function ({OwnerID:string,AwayID:string}):void} x
  * @param {function (string):void} y */
function fn(x, y) { }"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures4", content);
    fourslash::verify_no_errors(&mut s, );
}
