use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
}
