use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_of_anonymous_function() {
    let content = r#"(function [|foo|](): number {
    var x = [|foo|];
    return 0;
})"#;
    let mut s = Session::new_for_test("getOccurrencesOfAnonymousFunction", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
