use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_constructor_inheritance() {
    let content = r#"class base {
    constructor(s: string);
    constructor(n: number);
    constructor(a: any) { }
}
class B1 extends base { }
class B2 extends B1 { }
class B3 extends B2 {
    constructor() {
        super(/*indirectSuperCall*/3);
    }
}"#;
    let mut s = Session::new_for_test("signatureHelpConstructorInheritance", content);
    fourslash::go_to_marker(&mut s, "indirectSuperCall");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "B2(n: number): B2", ParameterCo
}
