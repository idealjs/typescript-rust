use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn proto_var_visible_with_outer_scope_underscore_proto() {
    let content = r#"// outer
var ___proto__ = 10;
function foo() {
    var __proto__ = "hello";
    /**/
}"#;
    let mut s = Session::new_for_test("protoVarVisibleWithOuterScopeUnderscoreProto", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
