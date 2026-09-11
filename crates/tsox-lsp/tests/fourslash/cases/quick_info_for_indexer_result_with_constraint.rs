use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_indexer_result_with_constraint() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo<T>(x: T) {
        return x;
}
function other2<T extends Date>(arg: T) {
    var b: { [x: string]: T };
    var /*1*/r2 = foo(b); // just shows T
}"#;
    let mut s = Session::new_for_test("quickInfoForIndexerResultWithConstraint", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local var) r2: {\n    [x: string]: T;\n}", "");
}
