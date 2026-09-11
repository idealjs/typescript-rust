use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_reference_from_link_tag2() {
    let content = r#"// @Filename: /a.ts
enum E {
    /** {@link /**/Foo} */
    Foo
}
interface Foo {
    foo: E.Foo;
}"#;
    let mut s = Session::new_for_test("renameReferenceFromLinkTag2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
