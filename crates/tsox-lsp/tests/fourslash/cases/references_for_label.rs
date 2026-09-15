use tsox_lsp::fourslash::Session;


#[test]
fn references_for_label() {
    let content = r#"/*1*/label: while (true) {
    if (false) /*2*/break /*3*/label;
    if (true) /*4*/continue /*5*/label;
}

/*6*/label: while (false) { }
var label = "label";"#;
    let _s = Session::new_for_test("referencesForLabel", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
