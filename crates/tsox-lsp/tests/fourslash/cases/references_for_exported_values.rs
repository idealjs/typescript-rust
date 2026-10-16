use tsox_lsp::fourslash::Session;


#[test]
fn references_for_exported_values() {
    let content = r#"namespace M {
    /*1*/export var /*2*/variable = 0;

    // local use
    var x = /*3*/variable;
}

// external use
M./*4*/variable"#;
    let _s = Session::new_for_test("referencesForExportedValues", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
