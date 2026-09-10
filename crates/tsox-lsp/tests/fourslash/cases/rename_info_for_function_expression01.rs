use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyRenameSucceeded"]
#[test]
fn rename_info_for_function_expression01() {
    let content = r#"var x = function /**/[|f|](g: any, h: any) {
    f(f, g);
}"#;
    let mut s = Session::new_for_test("renameInfoForFunctionExpression01", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
