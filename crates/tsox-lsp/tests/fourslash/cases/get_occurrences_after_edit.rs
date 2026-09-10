use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_after_edit() {
    let content = r#"/*0*/
interface A {
    foo: string;
}
function foo(x: A) {
    x.f/*1*/oo
}"#;
    let mut s = Session::new_for_test("getOccurrencesAfterEdit", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
    fourslash::go_to_marker(&mut s, "0");
    fourslash::insert(&mut s, "\n");
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
}
