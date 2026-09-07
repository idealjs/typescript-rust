use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_class_local() {
    let content = r#"var n = 14;

class foo {
    /*1*/private /*2*/n = 0;

    public bar() {
        this./*3*/n = 9;
    }

    constructor() {
        this./*4*/n = 4;
    }

    public bar2() {
        var n = 12;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
