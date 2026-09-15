use tsox_lsp::fourslash::Session;


#[test]
fn references_for_illegal_assignment() {
    let content = r#"f/*1*/oo = fo/*2*/o;
var /*bar*/bar = function () { };
bar = bar + 1;"#;
    let _s = Session::new_for_test("referencesForIllegalAssignment", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "bar")
}
