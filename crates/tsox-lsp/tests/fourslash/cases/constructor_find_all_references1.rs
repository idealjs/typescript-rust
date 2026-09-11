use tsox_lsp::fourslash::{self, Session};


#[test]
fn constructor_find_all_references1() {
    let content = r#"export class C {
    /**/public constructor() { }
    public foo() { }
}

new C().foo();"#;
    let mut s = Session::new_for_test("constructorFindAllReferences1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
