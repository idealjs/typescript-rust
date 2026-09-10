use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineVSFindAllReferences"]
#[test]
fn constructor_find_all_references1_vs() {
    let content = r#"export class C {
    /**/public constructor() { }
    public foo() { }
}

new C().foo();"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineVSFindAllReferences"); // f.VerifyBaselineVSFindAllReferences(t, "")
}
