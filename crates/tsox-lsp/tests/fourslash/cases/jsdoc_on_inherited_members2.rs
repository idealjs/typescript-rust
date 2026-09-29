use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_on_inherited_members2() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @filename: /a.js
/** @template T */
class A {
    /** Method documentation. */
    method() {}
}

/** @extends {A<number>} */
const B = class extends A {
    method() {}
}

const b = new B();
b.method/**/;"#;
    let _s = Session::new_for_test("jsdocOnInheritedMembers2", content);
    // TODO: f.VerifyBaselineHover(t)
}
