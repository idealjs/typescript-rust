use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_is_definition_of_computed_property() {
    let content = r#"let o = { /*1*/["/*2*/foo"]: 12 };
let y = o./*3*/foo;
let z = o['/*4*/foo'];"#;
    let _s = Session::new_for_test("getOccurrencesIsDefinitionOfComputedProperty", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
