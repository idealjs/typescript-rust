use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "any", "")
}
