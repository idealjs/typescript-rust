use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_on_type_predicates() {
    let content = r#"function f1(a: any): a is number {}
function f2<T>(a: any): a is T {}
function f3(a: any, ...b): a is number {}
f1(/*1*/)
f2(/*2*/)
f3(/*3*/)"#;
    let mut s = Session::new_for_test("signatureHelpOnTypePredicates", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f1(a: any): a is number"})
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f2(a: any): a is unknown"})
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f3(a: any, ...b: any[]): a is n
}
