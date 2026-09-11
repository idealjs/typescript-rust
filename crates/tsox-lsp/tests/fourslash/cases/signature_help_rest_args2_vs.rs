use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_rest_args2_vs() {
    let content = r#"// @strict: true
// @allowJs: true
// @checkJs: true
// @filename: index.js
const promisify = function (thisArg, fnName) {
    const fn = thisArg[fnName];
    return function () {
        return new Promise((resolve) => {
            fn.call(thisArg, ...arguments, /*1*/);
        });
    };
};"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
