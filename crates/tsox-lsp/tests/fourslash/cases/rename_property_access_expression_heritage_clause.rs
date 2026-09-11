use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_property_access_expression_heritage_clause() {
    let content = r#"class B {}
function foo() {
    return {[|[|{| "contextRangeIndex": 0 |}B|]: B|]};
}
class C extends (foo()).[|B|] {}
class C1 extends foo().[|B|] {}"#;
    let mut s = Session::new_for_test("renamePropertyAccessExpressionHeritageClause", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "B")
}
