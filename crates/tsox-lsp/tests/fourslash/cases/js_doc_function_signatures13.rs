use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_function_signatures13() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"/**
 * @template {string} K/**/ a golden opportunity
 */
function Multimap(iv) {
};"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures13", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "any", "")
}
