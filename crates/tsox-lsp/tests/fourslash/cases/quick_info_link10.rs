use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_link10() {
    let content = r#"/**
 * start {@link https://vscode.dev/ | end}
 */
const /**/a = () => 1;"#;
    let mut s = Session::new_for_test("quickInfoLink10", content);
    // TODO: f.VerifyBaselineHover(t)
}
