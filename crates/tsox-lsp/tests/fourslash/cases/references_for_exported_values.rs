use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_exported_values() {
    let content = r#"namespace M {
    /*1*/export var /*2*/variable = 0;

    // local use
    var x = /*3*/variable;
}

// external use
M./*4*/variable"#;
    let mut s = Session::new_for_test("referencesForExportedValues", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
