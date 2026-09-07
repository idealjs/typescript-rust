use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_function_check_type() {
    let content = r#"export type /**/Tail<T extends any[]> = ((...t: T) => void) extends (h: any, ...rest: infer R) => void ? R : never;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "type Tail<T extends any[]> = ((...t: T) => void) extends (h: any, ...res
}
