use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn arguments_index_expression() {
    let content = r#"function f() {
    var x = /**/arguments[0];
}"#;
    let mut s = Session::new_for_test("argumentsIndexExpression", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
