use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn code_fix_spelling_js3() {
    let content = r#"// @allowjs: true
// @noEmit: true
// @filename: a.js
class Classe {
    non = 'oui'
    methode() {
        // no error on 'this' references
        return this.none
    }
}
class Derivee extends Classe {
    methode() {
        // no error on 'super' references
        return super.none
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
