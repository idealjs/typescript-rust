use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_of_decorators() {
    let content = r#"// @Filename: b.ts
@/*1*/decorator
class C {
    @decorator
    method() {}
}
function decorator(target) {
    return target;
}"#;
    let mut s = Session::new_for_test("getOccurrencesOfDecorators", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
}
