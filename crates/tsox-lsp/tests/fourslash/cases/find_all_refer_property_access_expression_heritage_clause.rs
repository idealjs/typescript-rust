use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refer_property_access_expression_heritage_clause() {
    let content = r#"class B {}
function foo() {
    return {/*1*/B: B};
}
class C extends (foo())./*2*/B {}
class C1 extends foo()./*3*/B {}"#;
    let _s = Session::new_for_test("findAllReferPropertyAccessExpressionHeritageClause", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
