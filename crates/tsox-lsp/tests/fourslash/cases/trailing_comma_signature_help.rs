use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("trailingCommaSignatureHelp", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
