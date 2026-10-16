use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("callHierarchyJsxElement", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
