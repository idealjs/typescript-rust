use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_locations_for_function_expression01() {
    let content = r#"var x = [|function [|{| "contextRangeIndex": 0 |}f|](g: any, h: any) {
    [|f|]([|f|], g);
}|]"#;
    let mut s = Session::new_for_test("renameLocationsForFunctionExpression01", content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "f")
}
