use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
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
    let mut s = Session::new_for_test("jsdocLink1", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
