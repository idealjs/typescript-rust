use tsox_lsp::fourslash::Session;


#[test]
fn rename_function_parameter2() {
    let content = r#"/**
 * @param {number} p
 */
const foo = function foo(p/**/) {
    return p;
}"#;
    let _s = Session::new_for_test("renameFunctionParameter2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
