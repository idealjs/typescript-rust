use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_nested_type_argument_gt_balance() {
    let content = r#"declare function f<T, U>(): void;
type A<T> = T;
type B<T> = T;
type C<T> = T;
f<A<B<C<number>>>, /*nested*/;
"#;
    let mut s = Session::new_for_test("signatureHelpNestedTypeArgumentGTBalance", content);
    fourslash::go_to_marker(&mut s, "nested");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{
}
