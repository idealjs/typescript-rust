use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_at_before_space() {
    let content = r#"/**
 * @return Don't @ me
 */
function /*f*/f() { }
/**
 * @return One final @
 */
function /*g*/g() { }
/**
 * @return An @
 * But another line
 */
function /*h*/h() { }"#;
    let mut s = Session::new_for_test("quickInfoJSDocAtBeforeSpace", content);
    // TODO: f.VerifyBaselineHover(t)
}
