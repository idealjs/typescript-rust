use tsox_lsp::fourslash::{self, Session};


#[test]
fn assert_contextual_type() {
    let content = r#"<(aa: number) =>void >(function myFn(b/**/b) { });"#;
    let mut s = Session::new_for_test("assertContextualType", content);
    fourslash::verify_quick_info_at(&mut s, "", "(parameter) bb: number", "");
}
