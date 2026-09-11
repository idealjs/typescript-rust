use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions_ts_check() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
// @ts-check
interface I { a: number; b: number; }
interface J { b: number; c: number; }
declare const ij: I | J;
ij./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions_tsCheck", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["b"]);
}
