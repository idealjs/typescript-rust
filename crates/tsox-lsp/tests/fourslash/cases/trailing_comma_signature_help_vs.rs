use tsox_lsp::fourslash::{self, Session};


#[test]
fn trailing_comma_signature_help_vs() {
    let content = r#"function str(n: number): string;
/**
 * Stringifies a number with radix
 * @param radix The radix
 */
function str(n: number, radix: number): string;
function str(n: number, radix?: number): string { return ""; }

str(1, /*a*/)

declare function f<T>(a: T): T;
f(2, /*b*/);"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
