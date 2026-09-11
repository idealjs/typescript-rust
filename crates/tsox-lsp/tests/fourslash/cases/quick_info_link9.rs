use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_link9() {
    let content = r#"type Foo = {
    /**
     * Text before {@link /**/a} text after
     */
    c: (a: number) => void;
}"#;
    let mut s = Session::new_for_test("quickInfoLink9", content);
    // TODO: f.VerifyBaselineHover(t)
}
