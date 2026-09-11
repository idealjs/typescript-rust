use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_function_check_type() {
    let content = r#"export type /**/Tail<T extends any[]> = ((...t: T) => void) extends (h: any, ...rest: infer R) => void ? R : never;"#;
    let mut s = Session::new_for_test("quickInfoFunctionCheckType", content);
    fourslash::verify_quick_info_at(&mut s, "", "type Tail<T extends any[]> = ((...t: T) => void) extends (h: any, ...rest: infer R) => void ? R : never", "");
}
