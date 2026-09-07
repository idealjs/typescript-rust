use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_type_arguments() {
    let content = r#"declare function f(a: number, b: string, c: boolean): void; // ignored, not generic
declare function f<T extends number>(): void;
declare function f<T, U>(): void;
declare function f<T, U, V extends string>(): void;
f</*f0*/;
f<number, /*f1*/;
f<number, string, /*f2*/;

declare const C: {
    new<T extends number>(): void;
    new<T, U>(): void;
    new<T, U, V extends string>(): void;
};
new C</*C0*/;
new C<number, /*C1*/;
new C<number, string, /*C2*/;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "f0");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f<T extends number>(): void", P
    fourslash::go_to_marker(&mut s, "f1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f<T, U>(): void", ParameterName
    fourslash::go_to_marker(&mut s, "f2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f<T, U, V extends string>(): vo
    fourslash::go_to_marker(&mut s, "C0");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "C<T extends number>(): void", P
    fourslash::go_to_marker(&mut s, "C1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "C<T, U>(): void", ParameterName
    fourslash::go_to_marker(&mut s, "C2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "C<T, U, V extends string>(): vo
}
