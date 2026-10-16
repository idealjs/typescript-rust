use tsox_lsp::fourslash::{self, Session};


#[test]
fn range_format_starting_inside_js_doc_comment() {
    let content = r#"// @Filename: /a.ts
/**
 * @a
 * `
/*s*/ * @b
 */
export function f() {}/*e*/"#;
    let mut s = Session::new_for_test("rangeFormatStartingInsideJSDocComment", content);
    fourslash::format_selection(&mut s, "s", "e");
}
