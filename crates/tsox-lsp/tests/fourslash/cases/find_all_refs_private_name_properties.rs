use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_private_name_properties() {
    let content = r#"class C {
    /*1*/#foo = 10;
    constructor() {
        this./*2*/#foo = 20;
        /*3*/#foo in this;
    }
}
class D extends C {
    constructor() {
        super()
        this.#foo = 20;
    }
}
class E {
    /*4*/#foo: number;
    constructor() {
        this./*5*/#foo = 20;
    }
}"#;
    let mut s = Session::new_for_test("findAllRefsPrivateNameProperties", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
