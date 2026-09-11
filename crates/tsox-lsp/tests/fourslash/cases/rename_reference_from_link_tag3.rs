use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_reference_from_link_tag3() {
    let content = r#"// @filename: a.ts
interface Foo {
    foo: E.Foo;
}
// @Filename: b.ts
enum E {
    /** {@link /**/Foo} */
    Foo
}"#;
    let mut s = Session::new_for_test("renameReferenceFromLinkTag3", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
