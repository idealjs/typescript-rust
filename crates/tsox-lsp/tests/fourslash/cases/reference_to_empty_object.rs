use tsox_lsp::fourslash::{self, Session};


#[test]
fn reference_to_empty_object() {
    let content = r#"// @lib: es5
const obj = {}/*1*/;"#;
    let mut s = Session::new_for_test("referenceToEmptyObject", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
