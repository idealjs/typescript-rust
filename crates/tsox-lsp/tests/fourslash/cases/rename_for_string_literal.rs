use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_for_string_literal() {
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
    let mut s = Session::new_for_test("renameForStringLiteral", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
