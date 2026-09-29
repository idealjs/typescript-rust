use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_var_from_module_with_reopened_enums() {
    let content = r#"namespace A {
    /**/var o;
}
enum A {
}
enum A {
}
namespace A {
    var p;
}"#;
    let mut s = Session::new_for_test("removeVarFromModuleWithReopenedEnums", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.DeleteAtCaret(t, 6)
}
