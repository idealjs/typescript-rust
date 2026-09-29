use tsox_lsp::fourslash::{self, Session};


#[test]
fn module_enum_module() {
    let content = r#"namespace A {
    var o;
}
enum A {
    /**/c
}
namespace A {
    var p;
}"#;
    let mut s = Session::new_for_test("moduleEnumModule", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
}
