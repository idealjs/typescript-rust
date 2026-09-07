use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_link3() {
    let content = r#"class Foo<T> {
    /**
     * {@link Foo}
     * {@link Foo<T>}
     * {@link Foo<Array<X>>}
     * {@link Foo<>}
     * {@link Foo>}
     * {@link Foo<}
     */
    bar/**/(){}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
