use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_import_path() {
    let content = r#"// @Filename: /a.ts
export const x = 0;

// @Filename: /b.ts
import { x } from "[|./a|]";"#;
    let _s = Session::new_for_test("documentHighlightImportPath", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
