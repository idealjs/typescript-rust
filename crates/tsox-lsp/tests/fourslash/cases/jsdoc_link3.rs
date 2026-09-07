use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn jsdoc_link3() {
    let content = r#"// @Filename: /jsdocLink3.ts
export class C {
}
// @Filename: /module1.ts
import { C } from './jsdocLink3'
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
