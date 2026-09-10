use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_this() {
    let content = r#"class Foo<T> {
    public implicitAny(n: number) {
    }
    public explicitThis(this: this, n: number) {
        console.log(this);
    }
    public explicitClass(this: Foo<T>, n: number) {
        console.log(this);
    }
}

function implicitAny(x: number): void {
    return this;
}
function explicitVoid(this: void, x: number): void {
    return this;
}
function explicitLiteral(this: { n: number }, x: number): void {
    console.log(this);
}
let foo = new Foo<number>();
foo.implicitAny(/*1*/);
foo.explicitThis(/*2*/);
foo.explicitClass(/*3*/);
implicitAny(/*4*/12);
explicitVoid(/*5*/13);
let o = { n: 14, m: explicitLiteral };
o.m(/*6*/);"#;
    let mut s = Session::new_for_test("signatureHelpThis", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "n"})
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "n"})
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "n"})
    fourslash::go_to_marker(&mut s, "4");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "x"})
    fourslash::go_to_marker(&mut s, "5");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "x"})
    fourslash::go_to_marker(&mut s, "6");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "x"})
}
