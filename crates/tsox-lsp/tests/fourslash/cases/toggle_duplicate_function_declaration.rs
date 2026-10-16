use tsox_lsp::fourslash::{self, Session};


#[test]
fn toggle_duplicate_function_declaration() {
    let content = r#"class D { }
D();"#;
    let mut s = Session::new_for_test("toggleDuplicateFunctionDeclaration", content);
    fourslash::go_to_bof(&mut s, );
    fourslash::insert(&mut s, "declare function D();");
    fourslash::go_to_bof(&mut s, );
    // TODO: f.DeleteAtCaret(t, 21)
}
