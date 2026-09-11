use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_expando_like_property_with_setter_declaration_js1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
// @checkJs: true
// @filename: index.js
const x = {};

Object.defineProperty(x, "foo", {
  /** @param {number} v */
  set(v) {},
});

x.foo/**/ = 1;"#;
    let mut s = Session::new_for_test("quickInfoOnExpandoLikePropertyWithSetterDeclarationJs1", content);
    fourslash::verify_quick_info_at(&mut s, "", "(property) x.foo: number", "");
}
