use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
