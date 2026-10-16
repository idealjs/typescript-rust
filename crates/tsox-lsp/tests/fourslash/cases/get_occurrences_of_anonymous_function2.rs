use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_of_anonymous_function2() {
    let content = r#"//global foo definition
function foo() {}

(function f/*local*/oo(): number {
    return foo(); // local foo reference
})
//global foo references
fo/*global*/o();
var f = foo;"#;
    let _s = Session::new_for_test("getOccurrencesOfAnonymousFunction2", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "local", "global")
}
