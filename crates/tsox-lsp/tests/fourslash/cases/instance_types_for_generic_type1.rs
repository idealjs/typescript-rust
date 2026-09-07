use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn instance_types_for_generic_type1() {
    let content = r#"class G<T> {               // Introduce type parameter T
    self: G<T>;            // Use T as type argument to form instance type
    f() {
        this./*1*/self = /*2*/this;  // self and this are both of type G<T>
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) G<T>.self: G<T>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "this: this", "")
}
