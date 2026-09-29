use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_declare_in_module() {
    let content = r#"/**/export namespace Foo {
    function a(): void {}
}

Foo.a();"#;
    let mut s = Session::new_for_test("removeDeclareInModule", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.DeleteAtCaret(t, 7)
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
