use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_type_error() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"foo({
    /**/f: function() {},
    f() {}
});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(method) f(): void", "")
}
