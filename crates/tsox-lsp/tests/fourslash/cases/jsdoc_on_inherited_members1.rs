use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn jsdoc_on_inherited_members1() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: /a.js
/** @template T */
class A {
    /** Method documentation. */
    method() {}
}

/** @extends {A<number>} */
class B extends A {
    method() {}
}

const b = new B();
b.method/**/;"#;
    let mut s = Session::new_for_test("jsdocOnInheritedMembers1", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
