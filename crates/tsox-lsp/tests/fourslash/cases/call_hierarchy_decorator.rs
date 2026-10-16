use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_decorator() {
    let content = r#"// @experimentalDecorators: true
@bar
class Foo {
}

function /**/bar() {
    baz();
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyDecorator", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
