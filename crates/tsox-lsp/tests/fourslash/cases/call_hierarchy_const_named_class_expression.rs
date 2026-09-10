use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_const_named_class_expression() {
    let content = r#"function foo() {
    new Bar();
}

const /**/Bar = class {
    constructor() {
        baz();
    }
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyConstNamedClassExpression", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
