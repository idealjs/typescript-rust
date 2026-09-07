use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
