use tsox_lsp::fourslash::Session;


#[test]
fn rename_destructuring_function_parameter() {
    let content = r#"function f([|{[|{| "contextRangeIndex": 0 |}a|]}: {[|a|]}|]) {
    f({[|a|]});
}"#;
    let _s = Session::new_for_test("renameDestructuringFunctionParameter", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[2])
}
