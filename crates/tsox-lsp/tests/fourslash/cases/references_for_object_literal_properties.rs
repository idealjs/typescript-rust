use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_object_literal_properties() {
    let content = r#"var x = { /*1*/add: 0, b: "string" };
x["/*2*/add"];
x./*3*/add;
var y = x;
y./*4*/add;"#;
    let mut s = Session::new_for_test("referencesForObjectLiteralProperties", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
