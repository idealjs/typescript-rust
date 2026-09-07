use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn class_extends_interface_sig_help1() {
    let content = r#"class C {
    public foo(x: string);
    public foo(x: number);
    public foo(x: any) { return x; }
}
interface I extends C {
    other(x: any): any;
}
var i: I;
i.foo(/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterSpan: "x: string", OverloadsC
}
