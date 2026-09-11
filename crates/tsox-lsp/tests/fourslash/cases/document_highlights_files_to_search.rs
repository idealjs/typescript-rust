use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlights_files_to_search() {
    let content = r#"// @Filename: /a.ts
export const [|x|] = 0;
// @Filename: /b.ts
import { [|x|] } from "./a";"#;
    let mut s = Session::new_for_test("documentHighlights_filesToSearch", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
