use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_in_empty_file() {
    let content = r#"// @lib: es5
/*1*/"#;
    let mut s = Session::new_for_test("referencesInEmptyFile", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
