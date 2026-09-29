use tsox_lsp::fourslash::Session;


#[test]
fn references_for_label5() {
    let content = r#"/*1*/label:  while (true) {
            if (false) /*2*/break /*3*/label;
            function blah() {
/*4*/label:          while (true) {
                    if (false) /*5*/break /*6*/label;
                }
            }
            if (false) /*7*/break /*8*/label;
        }"#;
    let _s = Session::new_for_test("referencesForLabel5", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8")
}
