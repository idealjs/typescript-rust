use tsox_lsp::fourslash::{self, Session};


#[test]
fn static_generic_overloads1() {
    let content = r#"class A<T> {
    static B<S>(v: A<S>): A<S>;
    static B<S>(v: S): A<S>;
    static B<S>(v: any): A<S> {
        return null;
    }
}
var a = new A<number>();
A.B(/**/"#;
    let mut s = Session::new_for_test("staticGenericOverloads1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{OverloadsCount: 2})
    fourslash::insert(&mut s, "a");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(v: A<number>): A<number>", Ov
    fourslash::insert(&mut s, "); A.B(");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(v: A<unknown>): A<unknown>", 
    fourslash::insert(&mut s, "a");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(v: A<number>): A<number>", Ov
}
