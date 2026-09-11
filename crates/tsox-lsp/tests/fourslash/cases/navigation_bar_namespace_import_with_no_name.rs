use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_namespace_import_with_no_name() {
    let content = r#"import *{} from 'foo';"#;
    let mut s = Session::new_for_test("navigationBarNamespaceImportWithNoName", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
