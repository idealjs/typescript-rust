use tsox_lsp::fourslash::{self, Session};


#[test]
fn comments_union() {
    let content = r#"var a: Array<string> | Array<number>;
a./*1*/length"#;
    let mut s = Session::new_for_test("commentsUnion", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) Array<T>.length: number", "Gets or sets the length of the array. This is a number one higher than the highest index in the array.");
}
