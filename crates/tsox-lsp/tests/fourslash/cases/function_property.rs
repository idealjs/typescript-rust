use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "signatureA");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "x(a: number): void"})
    fourslash::go_to_marker(&mut s, "signatureB");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "x(a: number): void"})
    fourslash::go_to_marker(&mut s, "signatureC");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "x(a: number): void"})
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "completionA", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"completionB", "completionC"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "quickInfoA", "(method) x(a: number): void", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "quickInfoB",
        "(property) x: (a: number) => void",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "quickInfoC",
        "(property) x: (a: number) => void",
        "",
    );
}
