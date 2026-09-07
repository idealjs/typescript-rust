use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
