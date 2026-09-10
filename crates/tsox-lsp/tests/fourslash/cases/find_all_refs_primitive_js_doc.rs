use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_primitive_js_doc() {
    let content = r#"// @noLib: true
/**
 * @param {/*1*/number} n
 * @returns {/*2*/number}
 */
function f(n: /*3*/number): /*4*/number {}"#;
    let mut s = Session::new_for_test("findAllRefsPrimitiveJsDoc", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
