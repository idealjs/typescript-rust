use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_optional_param14() {
    let content = r#"function f(a: string): string;
function f(a: string, b: number): string;
function f(a: string, b?: number): string {
    return "";
}
f("", "", 1);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "addOptionalParam")
}
