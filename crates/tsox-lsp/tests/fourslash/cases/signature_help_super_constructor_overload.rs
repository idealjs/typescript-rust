use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_super_constructor_overload() {
    let content = r#"class SuperOverloadBase {
    constructor();
    constructor(test: string);
    constructor(test?: string) {
    }
}
class SuperOverLoad1 extends SuperOverloadBase {
    constructor() {
        super(/*superOverload1*/);
    }
}
class SuperOverLoad2 extends SuperOverloadBase {
    constructor() {
        super(""/*superOverload2*/);
    }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "superOverload1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "SuperOverloadBase(): SuperOverl
    fourslash::go_to_marker(&mut s, "superOverload2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "SuperOverloadBase(test: string)
}
