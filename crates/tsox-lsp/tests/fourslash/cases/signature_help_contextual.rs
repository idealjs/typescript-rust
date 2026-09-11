use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_contextual() {
    let content = r#"interface I {
    m(n: number, s: string): void;
    m2: () => void;
}
declare function takesObj(i: I): void;
takesObj({ m: (/*takesObj0*/) });
takesObj({ m(/*takesObj1*/) });
takesObj({ m: function(/*takesObj2*/) });
takesObj({ m2: (/*takesObj3*/) });

declare function takesCb(cb: (n: number, s: string, b: boolean) => void): void;
takesCb((/*contextualParameter1*/));
takesCb((/*contextualParameter1b*/) => {});
takesCb((n, /*contextualParameter2*/));
takesCb((n, s, /*contextualParameter3*/));
takesCb((n,/*contextualParameter3_2*/ s, b));
takesCb((n, s, b, /*contextualParameter4*/));

type Cb = () => void;
const cb: Cb = (/*contextualTypeAlias*/)

const cb2: () => void = (/*contextualFunctionType*/)"#;
    let mut s = Session::new_for_test("signatureHelp_contextual", content);
    fourslash::go_to_marker(&mut s, "takesObj0");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m(n: number, s: string): void",
    fourslash::go_to_marker(&mut s, "takesObj1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m(n: number, s: string): void",
    fourslash::go_to_marker(&mut s, "takesObj2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m(n: number, s: string): void",
    fourslash::go_to_marker(&mut s, "takesObj3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "m2(): void", ParameterCount: 0}
    fourslash::go_to_marker(&mut s, "contextualParameter1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "cb(n: number, s: string, b: boo
    fourslash::go_to_marker(&mut s, "contextualParameter1b");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "cb(n: number, s: string, b: boo
    fourslash::go_to_marker(&mut s, "contextualParameter2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "cb(n: number, s: string, b: boo
    fourslash::go_to_marker(&mut s, "contextualParameter3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "cb(n: number, s: string, b: boo
    fourslash::go_to_marker(&mut s, "contextualParameter3_2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "cb(n: number, s: string, b: boo
    fourslash::go_to_marker(&mut s, "contextualParameter4");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "cb(n: number, s: string, b: boo
    fourslash::go_to_marker(&mut s, "contextualTypeAlias");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Cb(): void", ParameterCount: 0}
    fourslash::go_to_marker(&mut s, "contextualFunctionType");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "cb2(): void", ParameterCount: 0
}
