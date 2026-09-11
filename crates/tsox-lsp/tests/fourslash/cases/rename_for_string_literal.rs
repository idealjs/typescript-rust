use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
