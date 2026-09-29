use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_from_link_tag_reference3() {
    let content = r#"// @filename: a.ts
interface Foo {
    foo: E.Foo;
}
// @Filename: b.ts
enum E {
    /** {@link /**/Foo} */
    Foo
}"#;
    let _s = Session::new_for_test("findAllReferencesFromLinkTagReference3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
