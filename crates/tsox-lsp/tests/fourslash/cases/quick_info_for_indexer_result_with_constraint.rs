use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(local var) r2: {\n    [x: string]: T;\n}", "")
}
