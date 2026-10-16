use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_from_link_tag_reference2() {
    let content = r#"// @Filename: /a.ts
enum E {
    /** {@link /**/Foo} */
    Foo
}
interface Foo {
    foo: E.Foo;
}"#;
    let _s = Session::new_for_test("findAllReferencesFromLinkTagReference2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
