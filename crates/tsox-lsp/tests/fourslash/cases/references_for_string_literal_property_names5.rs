use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_string_literal_property_names5() {
    let content = r#"var x = { "/*1*/someProperty": 0 }
x["/*2*/someProperty"] = 3;
x.someProperty = 5;"#;
    let mut s = Session::new_for_test("referencesForStringLiteralPropertyNames5", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
