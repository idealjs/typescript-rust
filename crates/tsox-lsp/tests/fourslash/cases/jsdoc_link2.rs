use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_link2() {
    let content = r#"// @Filename: jsdocLink2.ts
class C {
}
// @Filename: script.ts
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
    let mut s = Session::new_for_test("jsdocLink2", content);
    // TODO: f.VerifyBaselineHover(t)
}
