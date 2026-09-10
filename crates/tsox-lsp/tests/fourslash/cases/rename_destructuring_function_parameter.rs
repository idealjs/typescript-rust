use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_destructuring_function_parameter() {
    let content = r#"function f([|{[|{| "contextRangeIndex": 0 |}a|]}: {[|a|]}|]) {
    f({[|a|]});
}"#;
    let mut s = Session::new_for_test("renameDestructuringFunctionParameter", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3], f.Ranges()[2])
}
