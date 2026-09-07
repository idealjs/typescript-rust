use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineCallHierarchy"]
#[test]
fn call_hierarchy_jsx_element() {
    let content = r#"// @jsx: preserve
// @filename: main.tsx
function foo() {
    return <Bar/>;
}

function /**/Bar() {
    baz();
}

function baz() {
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyBaselineCallHierarchy"); // f.VerifyBaselineCallHierarchy(t)
}
