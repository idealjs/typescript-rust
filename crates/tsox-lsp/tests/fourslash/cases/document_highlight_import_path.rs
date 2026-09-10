use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_import_path() {
    let content = r#"// @Filename: /a.ts
export const x = 0;

// @Filename: /b.ts
import { x } from "[|./a|]";"#;
    let mut s = Session::new_for_test("documentHighlightImportPath", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
