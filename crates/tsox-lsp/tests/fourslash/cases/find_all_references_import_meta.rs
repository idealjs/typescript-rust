use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_references_import_meta() {
    let content = r#"// Haha that's so meta!

let x = import.meta/**/;"#;
    let mut s = Session::new_for_test("findAllReferencesImportMeta", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
