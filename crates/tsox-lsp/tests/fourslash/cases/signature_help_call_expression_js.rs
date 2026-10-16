use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn signature_help_call_expression_js() {
    let content = r#"// @strict: false
// @checkJs: true
// @allowJs: true
// @Filename: main.js
function allOptional() { arguments; }
allOptional(/*1*/);
allOptional(1, 2, 3);
function someOptional(x, y) { arguments; }
someOptional(/*2*/);
someOptional(1, 2, 3);
someOptional(); // no error here; x and y are optional in JS"#;
    let mut s = Session::new_for_test("signatureHelpCallExpressionJs", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "allOptional(...args: any[]): vo
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "someOptional(x: any, y: any, ..
}
