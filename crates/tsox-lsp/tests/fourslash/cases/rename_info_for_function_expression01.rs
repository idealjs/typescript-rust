use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_info_for_function_expression01() {
    let content = r#"var x = function /**/[|f|](g: any, h: any) {
    f(f, g);
}"#;
    let mut s = Session::new_for_test("renameInfoForFunctionExpression01", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
