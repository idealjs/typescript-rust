use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
    fourslash::go_to_marker(&mut s, "0");
    fourslash::insert(&mut s, "\n");
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1")
}
