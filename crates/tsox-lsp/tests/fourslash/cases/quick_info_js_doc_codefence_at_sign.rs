use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_codefence_at_sign() {
    let content = r#"/**
 * text
 * @example Foo
 * ```
 * @Embed[asfasdfasf]
 * ```
 * becomes
 * ```html
 * <div></div>
 * ```
 */
const /*1*/x = 1;

/**
 * Some text
 * ```
 * @tag inside code
 * ```
 * @param y - a number
 */
function /*2*/foo(y: number) {}
"#;
    let mut s = Session::new_for_test("quickInfoJSDocCodefenceAtSign", content);
    // TODO: f.VerifyBaselineHover(t)
}
