use tsox_lsp::fourslash::{self, Session};


#[test]
fn add_member_to_module() {
    let content = r#"namespace A {
    /*var*/
}
module /*check*/A {
    var p;
}"#;
    let mut s = Session::new_for_test("addMemberToModule", content);
    fourslash::go_to_marker(&mut s, "check");
    // TODO: f.VerifyQuickInfoExists(t)
    fourslash::go_to_marker(&mut s, "var");
    fourslash::insert(&mut s, "var o;");
    fourslash::go_to_marker(&mut s, "check");
    // TODO: f.VerifyQuickInfoExists(t)
}
