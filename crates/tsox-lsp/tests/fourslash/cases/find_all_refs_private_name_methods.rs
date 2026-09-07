use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_private_name_methods() {
    let content = r#"class C {
    /*1*/#foo(){ }
    constructor() {
        this./*2*/#foo();
    }
}
class D extends C {
    constructor() {
        super()
        this.#foo = 20;
    }
}
class E {
    /*3*/#foo(){ }
    constructor() {
        this./*4*/#foo();
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
