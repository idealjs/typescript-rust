use tsox_lsp::fourslash::{self, Session};


#[test]
fn constructor_find_all_references2() {
    let content = r#"export class C {
    /**/private constructor() { }
    public foo() { }
}

new C().foo();"#;
    let mut s = Session::new_for_test("constructorFindAllReferences2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
