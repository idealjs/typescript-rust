use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_template_tag() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /foo.js
/**
 * Doc
 * @template {new (...args: any[]) => any} T
 * @param {T} cls
 */
function /**/myMixin(cls) {
    return class extends cls {}
}"#;
    let mut s = Session::new_for_test("quickInfoTemplateTag", content);
    fourslash::verify_quick_info_at(&mut s, "", "function myMixin<T extends new (...args: any[]) => any>(cls: T): {\n    new (...args: any[]): (Anonymous class);\n    prototype: myMixin<any>.(Anonymous class);\n} & T", "Doc");
}
