use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_rest_args2() {
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
