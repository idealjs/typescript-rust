use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn get_occurrences_is_definition_of_computed_property() {
    let content = r#"let o = { /*1*/["/*2*/foo"]: 12 };
let y = o./*3*/foo;
let z = o['/*4*/foo'];"#;
    let mut s = Session::new_for_test("getOccurrencesIsDefinitionOfComputedProperty", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
