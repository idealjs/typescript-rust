use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_call_expression_by_const_named_function_expression() {
    let content = r#"function foo() {
    bar();
}

const bar = function () {
    baz();
}

function baz() {
}

/**/bar()"#;
    let mut s = Session::new_for_test("callHierarchyCallExpressionByConstNamedFunctionExpression", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
