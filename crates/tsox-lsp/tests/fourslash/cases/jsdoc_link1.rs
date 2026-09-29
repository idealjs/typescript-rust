use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_link1() {
    let content = r#"class C {
}
/**
 * {@link C}
 * @wat Makes a {@link C}. A default one.
 * {@link C()}
 * {@link C|postfix text}
 * {@link unformatted postfix text}
 * @see {@link C} its great
 */
function /**/CC() {
}"#;
    let _s = Session::new_for_test("jsdocLink1", content);
    // TODO: f.VerifyBaselineHover(t)
}
