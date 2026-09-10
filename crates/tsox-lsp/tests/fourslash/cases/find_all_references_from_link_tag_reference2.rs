use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
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
    let mut s = Session::new_for_test("findAllReferencesFromLinkTagReference2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
