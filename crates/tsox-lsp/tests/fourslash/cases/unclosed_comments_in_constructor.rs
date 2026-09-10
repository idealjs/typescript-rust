use tsox_lsp::fourslash::{self, Session};


#[test]
fn unclosed_comments_in_constructor() {
    let content = r#"class Foo {
    constructor(/* /**/) { }
}"#;
    let mut s = Session::new_for_test("unclosedCommentsInConstructor", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
