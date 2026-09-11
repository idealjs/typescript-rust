use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_expando_like_property_with_setter_declaration_js2() {
    let content = r#"// @strict: true
// @checkJs: true
// @filename: index.js
const obj = {};
let val = 10;
Object.defineProperty(obj, "a", {
  configurable: true,
  enumerable: true,
  set(v) {
    val = v;
  },
});

obj.a/**/ = 100;"#;
    let mut s = Session::new_for_test("quickInfoOnExpandoLikePropertyWithSetterDeclarationJs2", content);
    fourslash::verify_quick_info_at(&mut s, "", "(property) obj.a: any", "");
}
