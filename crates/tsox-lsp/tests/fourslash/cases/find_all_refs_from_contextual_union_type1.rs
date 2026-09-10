use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_from_contextual_union_type1() {
    let content = r#"// @strict: true
function test1(arg: { prop: "foo" }) {}
test1({ /*1*/prop: "bar" });

function test2(arg: { prop: "foo" } | undefined) {}
test2({ /*2*/prop: "bar" });"#;
    let mut s = Session::new_for_test("findAllRefsFromContextualUnionType1", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
