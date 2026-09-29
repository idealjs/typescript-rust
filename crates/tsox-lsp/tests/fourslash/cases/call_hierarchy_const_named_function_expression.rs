use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_const_named_function_expression() {
    let content = r#"function foo() {
    bar();
}

const /**/bar = function () {
    baz();
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyConstNamedFunctionExpression", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
