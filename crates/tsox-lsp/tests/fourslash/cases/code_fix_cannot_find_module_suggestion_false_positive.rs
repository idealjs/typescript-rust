use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn code_fix_cannot_find_module_suggestion_false_positive() {
    let content = r#"// @moduleResolution: bundler
// @module: commonjs
// @resolveJsonModule: true
// @strict: true
// @Filename: /node_modules/foo/bar.json
{ "a": 0 }
// @Filename: /a.ts
import abs = require([|"foo/bar.json"|]);
abs;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_file(&mut s, "/a.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}
