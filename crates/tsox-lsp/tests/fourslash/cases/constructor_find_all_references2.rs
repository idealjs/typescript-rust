use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn constructor_find_all_references2() {
    let content = r#"export class C {
    /**/private constructor() { }
    public foo() { }
}

new C().foo();"#;
    let mut s = Session::new_for_test("constructorFindAllReferences2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
