use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn rename_property_access_expression_heritage_clause() {
    let content = r#"class B {}
function foo() {
    return {[|[|{| "contextRangeIndex": 0 |}B|]: B|]};
}
class C extends (foo()).[|B|] {}
class C1 extends foo().[|B|] {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "B")
}
