use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_string_literal_property_names6() {
    let content = r#"const x = function () { return 111111; }
x./*1*/someProperty = 5;
x["/*2*/someProperty"] = 3;"#;
    let mut s = Session::new_for_test("referencesForStringLiteralPropertyNames6", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
