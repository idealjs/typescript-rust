use tsox_lsp::fourslash::{self, Session};


#[test]
fn automatic_constructor_toggling() {
    let content = r#"class A<T> { }
class B<T> {/*B*/ }
class C<T> { /*C*/constructor(val: T) { } }
class D<T> { constructor(/*D*/val: T) { } }

new /*Asig*/A<string>();
new /*Bsig*/B("");
new /*Csig*/C("");
new /*Dsig*/D<string>();"#;
    let mut s = Session::new_for_test("automaticConstructorToggling", content);
    fourslash::go_to_marker(&mut s, "B");
    fourslash::insert(&mut s, "constructor(val: T) { }");
    fourslash::verify_quick_info_at(&mut s, "Asig", "constructor A<string>(): A<string>", "");
    fourslash::verify_quick_info_at(&mut s, "Bsig", "constructor B<string>(val: string): B<string>", "");
    fourslash::verify_quick_info_at(&mut s, "Csig", "constructor C<string>(val: string): C<string>", "");
    fourslash::verify_quick_info_at(&mut s, "Dsig", "constructor D<string>(val: string): D<string>", "");
    fourslash::go_to_marker(&mut s, "C");
    fourslash::delete_at_caret(&mut s, 23);
    fourslash::verify_quick_info_at(&mut s, "Asig", "constructor A<string>(): A<string>", "");
    fourslash::verify_quick_info_at(&mut s, "Bsig", "constructor B<string>(val: string): B<string>", "");
    fourslash::verify_quick_info_at(&mut s, "Csig", "constructor C<unknown>(): C<unknown>", "");
    fourslash::verify_quick_info_at(&mut s, "Dsig", "constructor D<string>(val: string): D<string>", "");
    fourslash::go_to_marker(&mut s, "D");
    fourslash::delete_at_caret(&mut s, 6);
    fourslash::verify_quick_info_at(&mut s, "Asig", "constructor A<string>(): A<string>", "");
    fourslash::verify_quick_info_at(&mut s, "Bsig", "constructor B<string>(val: string): B<string>", "");
    fourslash::verify_quick_info_at(&mut s, "Csig", "constructor C<unknown>(): C<unknown>", "");
    fourslash::verify_quick_info_at(&mut s, "Dsig", "constructor D<string>(): D<string>", "");
}
