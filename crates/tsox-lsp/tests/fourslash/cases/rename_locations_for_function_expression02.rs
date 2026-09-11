use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_locations_for_function_expression02() {
    let content = r#"function f() {

}
var x = [|function [|{| "contextRangeIndex": 0 |}f|](g: any, h: any) {

    let helper = function f(): any { f(); }

    let foo = () => [|f|]([|f|], g);
}|]"#;
    let mut s = Session::new_for_test("renameLocationsForFunctionExpression02", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "f")
}
