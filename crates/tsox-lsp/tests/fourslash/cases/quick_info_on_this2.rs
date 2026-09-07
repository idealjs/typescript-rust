use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "this: this", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "this: Bar<T>", "")
}
