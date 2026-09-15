use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_link11() {
    let content = r#"/**
 * {@link https://vscode.dev}
 * [link text]{https://vscode.dev}
 * {@link https://vscode.dev|link text}
 * {@link https://vscode.dev link text}
 */
function f() {}

/**/f();"#;
    let _s = Session::new_for_test("quickInfoLink11", content);
    // TODO: f.VerifyBaselineHover(t)
}
