use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_class() {
    let content = r#"function foo() {
    bar();
}

function /**/bar() {
    new Baz();
}

class Baz {
}"#;
    let mut s = Session::new_for_test("callHierarchyClass", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
