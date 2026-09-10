use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_for_super_calls1() {
    let content = r#"class A { }
class B extends A { }
class C extends B {
    constructor() {
        super(/*1*/ // sig help here?
    }
}
class A2 { }
class B2 extends A2 {
    constructor(x:number) {}
 }
class C2 extends B2 {
    constructor() {
        super(/*2*/ // sig help here?
    }
}"#;
    let mut s = Session::new_for_test("signatureHelpForSuperCalls1", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B(): B"})
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B2(x: number): B2"})
}
