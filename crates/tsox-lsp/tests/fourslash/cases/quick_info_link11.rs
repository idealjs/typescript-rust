use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
