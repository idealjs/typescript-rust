use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_tagged_template() {
    let content = r#"function foo() {
    bar`a${1}b`;
}

function /**/bar(array: TemplateStringsArray, ...args: any[]) {
    baz();
}

function baz() {
}"#;
    let mut s = Session::new_for_test("callHierarchyTaggedTemplate", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
