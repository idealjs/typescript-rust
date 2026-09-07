use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn trailing_comma_signature_help() {
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
