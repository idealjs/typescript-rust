use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoLink3", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineHover(t)
}
