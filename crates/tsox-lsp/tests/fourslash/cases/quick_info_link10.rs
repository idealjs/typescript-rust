use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_link10() {
    let content = r#"/**
 * start {@link https://vscode.dev/ | end}
 */
const /**/a = () => 1;"#;
    let _s = Session::new_for_test("quickInfoLink10", content);
    // TODO: f.VerifyBaselineHover(t)
}
