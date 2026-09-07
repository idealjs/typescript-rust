use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn proto_var_visible_with_outer_scope_underscore_proto() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// outer
var ___proto__ = 10;
function foo() {
    var __proto__ = "hello";
    /**/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
