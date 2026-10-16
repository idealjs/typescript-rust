use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_accessor() {
    let content = r#"function foo() {
    new C().bar;
}

class C {
    get /**/bar() {
        return baz();
    }
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyAccessor", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
