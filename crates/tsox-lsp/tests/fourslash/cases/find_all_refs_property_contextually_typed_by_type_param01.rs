use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_property_contextually_typed_by_type_param01() {
    let content = r#"interface IFoo {
    /*1*/a: string;
}
class C<T extends IFoo> {
    method() {
        var x: T = {
            a: ""
        };
        x.a;
    }
}


var x: IFoo = {
    a: "ss"
};"#;
    let mut s = Session::new_for_test("findAllRefsPropertyContextuallyTypedByTypeParam01", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
