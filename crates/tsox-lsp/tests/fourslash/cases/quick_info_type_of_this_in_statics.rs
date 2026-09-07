use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_type_of_this_in_statics() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class C {
    static foo() {
        var /*1*/r = this;
    }
    static get x() {
        var /*2*/r = this;
        return 1;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local var) r: typeof C", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(local var) r: typeof C", "");
}
