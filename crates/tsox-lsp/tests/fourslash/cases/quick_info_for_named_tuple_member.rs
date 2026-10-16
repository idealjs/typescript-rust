use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_named_tuple_member() {
    let content = r#"type foo = [/**/x: string];"#;
    let mut s = Session::new_for_test("quickInfoForNamedTupleMember", content);
    fourslash::verify_quick_info_at(&mut s, "", "string", "");
}
