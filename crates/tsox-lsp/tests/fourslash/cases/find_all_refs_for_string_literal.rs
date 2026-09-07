use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
