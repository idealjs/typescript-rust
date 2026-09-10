use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_non_string_import_attributes() {
    let content = r#"// @module: node18
import * as react from "react" with { cache: /**/0 };
react.Children;"#;
    let mut s = Session::new_for_test("getOccurrencesNonStringImportAttributes", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
