use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_incomplete_calls() {
    let content = r#"namespace IncompleteCalls {
    class Foo {
        public f1() { }
        public f2(n: number): number { return 0; }
        public f3(n: number, s: string) : string { return ""; }
    }
    var x = new Foo();
    x.f1();
    x.f2(5);
    x.f3(5, "");
    x.f1(/*incompleteCalls1*/
    x.f2(5,/*incompleteCalls2*/
    x.f3(5,/*incompleteCalls3*/
}"#;
    let mut s = Session::new_for_test("signatureHelpIncompleteCalls", content);
    fourslash::go_to_marker(&mut s, "incompleteCalls1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f1(): void", ParameterCount: 0}
    fourslash::go_to_marker(&mut s, "incompleteCalls2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f2(n: number): number", Paramet
    fourslash::go_to_marker(&mut s, "incompleteCalls3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f3(n: number, s: string): strin
}
