use tsox_lsp::fourslash::{self, Session};


#[test]
fn call_hierarchy_function_ambiguity2() {
    let content = r#"// @filename: a.d.ts
declare function /**/foo(x?: number): void;
// @filename: b.d.ts
declare function foo(x?: string): void;
declare function foo(x?: boolean): void;
// @filename: main.ts
function bar() {
    foo();
}"#;
    let mut s = Session::new_for_test("callHierarchyFunctionAmbiguity2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyBaselineCallHierarchy(t)
}
