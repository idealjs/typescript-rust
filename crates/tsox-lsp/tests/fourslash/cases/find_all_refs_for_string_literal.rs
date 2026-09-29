use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_string_literal() {
    let content = r#"// @filename: /a.ts
interface Foo {
    property: /**/"foo";
}
/**
 * @type {{ property: "foo"}}
 */
const obj: Foo = {
    property: "foo",
}"#;
    let _s = Session::new_for_test("findAllRefsForStringLiteral", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
