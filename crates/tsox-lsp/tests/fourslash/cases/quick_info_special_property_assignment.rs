use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_special_property_assignment() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: /a.js
class C {
    constructor() {
      /** Doc */
      this./*write*/x = 0;
      this./*read*/x;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "write", "(property) C.x: any", "Doc");
    fourslash::verify_quick_info_at(&mut s, "read", "(property) C.x: number", "Doc");
}
