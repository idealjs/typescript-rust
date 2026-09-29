use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_function() {
    let content = r#"function foo() {
    bar();
}

function /**/bar() {
    baz();
    quxx();
    baz();
}

function baz() {
}

function quxx() {
}"#;
    let mut s = Session::new_for_test("callHierarchyFunction", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
