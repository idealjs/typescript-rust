use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn signature_help_call_expression_js() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "allOptional(...args: any[]): vo
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "someOptional(x: any, y: any, ..
}
