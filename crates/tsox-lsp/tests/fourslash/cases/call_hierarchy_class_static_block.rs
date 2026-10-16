use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_class_static_block() {
    let content = r#"class C {
    static {
        function foo() {
            bar();
        }

        function /**/bar() {
            baz();
            quxx();
            baz();
        }

        foo();
    }
}

function baz() {
}

function quxx() {
}"#;
    let mut s = Session::new_for_test("callHierarchyClassStaticBlock", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
