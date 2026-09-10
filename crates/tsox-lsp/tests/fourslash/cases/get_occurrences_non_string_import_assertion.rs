use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_non_string_import_assertion() {
    let content = r#"// @module: node18
import * as react from "react" with { cache: /**/0 };
react.Children;"#;
    let mut s = Session::new_for_test("getOccurrencesNonStringImportAssertion", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
