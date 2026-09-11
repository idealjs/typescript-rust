use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_private_name_accessors() {
    let content = r#"class C {
    /*1*/get /*2*/#foo(){ return 1; }
    /*3*/set /*4*/#foo(value: number){  }
    constructor() {
        this./*5*/#foo();
    }
}
class D extends C {
    constructor() {
        super()
        this.#foo = 20;
    }
}
class E {
    /*6*/get /*7*/#foo(){ return 1; }
    /*8*/set /*9*/#foo(value: number){  }
    constructor() {
        this./*10*/#foo();
    }
}"#;
    let mut s = Session::new_for_test("findAllRefsPrivateNameAccessors", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8", "9", "10")
}
