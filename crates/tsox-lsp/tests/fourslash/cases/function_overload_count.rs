use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn function_overload_count() {
    let content = r#"class C1 {
    public attr(): string;
    public attr(i: number): string;
    public attr(i: number, x: boolean): string;
    public attr(i?: any, x?: any) {
        return "hi";
    }
}
var i = new C1;
i.attr(/*1*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{OverloadsCount: 3})
}
