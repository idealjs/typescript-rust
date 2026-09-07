use tsox_lsp::fourslash::{self, Session};

#[test]
fn instance_types_for_generic_type1() {
    let content = r#"class G<T> {               // Introduce type parameter T
    self: G<T>;            // Use T as type argument to form instance type
    f() {
        this./*1*/self = /*2*/this;  // self and this are both of type G<T>
    }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) G<T>.self: G<T>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "this: this", "");
}
