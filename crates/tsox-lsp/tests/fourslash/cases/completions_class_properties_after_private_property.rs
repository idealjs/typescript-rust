use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_class_properties_after_private_property() {
    let content = r#"interface X {
    bla: string;
}
class Y implements X {
    private blub = "";
    /**/
}"#;
    let mut s = Session::new_for_test("completionsClassPropertiesAfterPrivateProperty", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["bla"], &[]);
}
