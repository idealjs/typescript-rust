use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_in() {
    let content = r#"var obj;
for (var /**/p in obj) { }"#;
    let mut s = Session::new_for_test("quickInfoForIn", content);
    fourslash::verify_quick_info_at(&mut s, "", "var p: string", "");
}
