use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_link9() {
    let content = r#"type Foo = {
    /**
     * Text before {@link /**/a} text after
     */
    c: (a: number) => void;
}"#;
    let _s = Session::new_for_test("quickInfoLink9", content);
    // TODO: f.VerifyBaselineHover(t)
}
