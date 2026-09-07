use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_class_static_block2() {
    let content = r#"class C {
    /**/static {
        function foo() {
            bar();
        }

        function bar() {
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
