use tsox_lsp::fourslash::{self, Session};


#[test]
fn function_property() {
    let content = r#"var a = {
    x(a: number) { }
};

var b = {
    x: function (a: number) { }
};

var c = {
    x: (a: number) => { }
};
a.x(/*signatureA*/1);
b.x(/*signatureB*/1);
c.x(/*signatureC*/1);
a./*completionA*/;
b./*completionB*/;
c./*completionC*/;
a./*quickInfoA*/x;
b./*quickInfoB*/x;
c./*quickInfoC*/x;"#;
    let mut s = Session::new_for_test("functionProperty", content);
    fourslash::go_to_marker(&mut s, "signatureA");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "x(a: number): void"})
    fourslash::go_to_marker(&mut s, "signatureB");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "x(a: number): void"})
    fourslash::go_to_marker(&mut s, "signatureC");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "x(a: number): void"})
    // TODO: f.VerifyCompletions(t, "completionA", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"completionB", "completionC"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "quickInfoA", "(method) x(a: number): void", "");
    fourslash::verify_quick_info_at(&mut s, "quickInfoB", "(property) x: (a: number) => void", "");
    fourslash::verify_quick_info_at(&mut s, "quickInfoC", "(property) x: (a: number) => void", "");
}
