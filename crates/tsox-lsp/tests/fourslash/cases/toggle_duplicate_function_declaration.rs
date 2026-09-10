use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn toggle_duplicate_function_declaration() {
    let content = r#"class D { }
D();"#;
    let mut s = Session::new_for_test("toggleDuplicateFunctionDeclaration", content);
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::insert(&mut s, "declare function D();");
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 21)
}
