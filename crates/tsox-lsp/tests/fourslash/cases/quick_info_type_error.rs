use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_type_error() {
    let content = r#"foo({
    /**/f: function() {},
    f() {}
});"#;
    let mut s = Session::new_for_test("quickInfoTypeError", content);
    fourslash::verify_quick_info_at(&mut s, "", "(method) f(): void", "");
}
