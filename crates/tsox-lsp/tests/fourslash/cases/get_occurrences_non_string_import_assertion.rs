use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_non_string_import_assertion() {
    let content = r#"// @module: node18
import * as react from "react" with { cache: /**/0 };
react.Children;"#;
    let _s = Session::new_for_test("getOccurrencesNonStringImportAssertion", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
