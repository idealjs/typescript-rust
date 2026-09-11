use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlights_invalid_modifier_locations() {
    let content = r#"class C {
    m([|readonly|] p) {}
}
function f([|readonly|] p) {}

class D {
    m([|public|] p) {}
}
function g([|public|] p) {}"#;
    let mut s = Session::new_for_test("documentHighlightsInvalidModifierLocations", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
