use tsox_lsp::fourslash::{self, Session};


#[test]
fn constructor_find_all_references4() {
    let content = r#"export class C {
    /**/protected constructor() { }
    public foo() { }
}

new C().foo();"#;
    let mut s = Session::new_for_test("constructorFindAllReferences4", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
