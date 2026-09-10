use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_label() {
    let content = r#"/*1*/label: while (true) {
    if (false) /*2*/break /*3*/label;
    if (true) /*4*/continue /*5*/label;
}

/*6*/label: while (false) { }
var label = "label";"#;
    let mut s = Session::new_for_test("referencesForLabel", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
