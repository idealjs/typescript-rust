use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refer_property_access_expression_heritage_clause() {
    let content = r#"class B {}
function foo() {
    return {/*1*/B: B};
}
class C extends (foo())./*2*/B {}
class C1 extends foo()./*3*/B {}"#;
    let mut s = Session::new_for_test("findAllReferPropertyAccessExpressionHeritageClause", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
