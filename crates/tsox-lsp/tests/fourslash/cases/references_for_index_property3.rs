use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_index_property3() {
    let content = r#"interface Object {
    /*1*/toMyString();
}

var y: Object;
y./*2*/toMyString();

var x = {};
x["/*3*/toMyString"]();"#;
    let mut s = Session::new_for_test("referencesForIndexProperty3", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
