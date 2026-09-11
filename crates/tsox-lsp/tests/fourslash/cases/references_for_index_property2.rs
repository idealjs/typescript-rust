use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_index_property2() {
    let content = r#"var a;
a["/*1*/blah"];"#;
    let mut s = Session::new_for_test("referencesForIndexProperty2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
