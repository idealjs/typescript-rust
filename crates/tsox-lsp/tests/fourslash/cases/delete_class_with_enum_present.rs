use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn delete_class_with_enum_present() {
    let content = r#"enum Foo { a, b, c }
/**/class Bar { }"#;
    let mut s = Session::new_for_test("deleteClassWithEnumPresent", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 13)
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
