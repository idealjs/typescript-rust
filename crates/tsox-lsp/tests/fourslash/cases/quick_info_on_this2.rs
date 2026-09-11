use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_this2() {
    let content = r#"class Bar<T> {
    public explicitThis(this: this) {
        console.log(th/*1*/is);
    }
    public explicitClass(this: Bar<T>) {
        console.log(thi/*2*/s);
    }
}"#;
    let mut s = Session::new_for_test("quickInfoOnThis2", content);
    fourslash::verify_quick_info_at(&mut s, "1", "this: this", "");
    fourslash::verify_quick_info_at(&mut s, "2", "this: Bar<T>", "");
}
