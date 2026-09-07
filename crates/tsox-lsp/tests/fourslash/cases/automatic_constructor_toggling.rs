use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "B");
    fourslash::insert(&mut s, "constructor(val: T) { }");
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Asig", "constructor A<string>(): A<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Bsig", "constructor B<string>(val: string): B<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Csig", "constructor C<string>(val: string): C<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Dsig", "constructor D<string>(val: string): D<string>", "")
    fourslash::go_to_marker(&mut s, "C");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 23)
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Asig", "constructor A<string>(): A<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Bsig", "constructor B<string>(val: string): B<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Csig", "constructor C<unknown>(): C<unknown>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Dsig", "constructor D<string>(val: string): D<string>", "")
    fourslash::go_to_marker(&mut s, "D");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 6)
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Asig", "constructor A<string>(): A<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Bsig", "constructor B<string>(val: string): B<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Csig", "constructor C<unknown>(): C<unknown>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "Dsig", "constructor D<string>(): D<string>", "")
}
