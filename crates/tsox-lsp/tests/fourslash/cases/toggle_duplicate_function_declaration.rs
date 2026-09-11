use tsox_lsp::fourslash::{self, Session};


#[test]
fn toggle_duplicate_function_declaration() {
    let content = r#"class D { }
D();"#;
    let mut s = Session::new_for_test("toggleDuplicateFunctionDeclaration", content);
    // TODO: f.GoToBOF(t)
    fourslash::insert(&mut s, "declare function D();");
    // TODO: f.GoToBOF(t)
    // TODO: f.DeleteAtCaret(t, 21)
}
