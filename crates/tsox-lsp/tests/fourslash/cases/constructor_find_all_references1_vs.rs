use tsox_lsp::fourslash::{self, Session};


#[test]
fn constructor_find_all_references1_vs() {
    let content = r#"export class C {
    /**/public constructor() { }
    public foo() { }
}

new C().foo();"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineVSFindAllReferences(t, "")
}
