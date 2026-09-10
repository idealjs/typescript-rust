use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_const_named_arrow_function() {
    let content = r#"function foo() {
    bar();
}

const /**/bar = () => {
    baz();
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyConstNamedArrowFunction", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
