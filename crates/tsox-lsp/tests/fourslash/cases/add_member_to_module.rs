use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn add_member_to_module() {
    let content = r#"namespace A {
    /*var*/
}
module /*check*/A {
    var p;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "check");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "var");
    fourslash::insert(&mut s, "var o;");
    fourslash::go_to_marker(&mut s, "check");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
