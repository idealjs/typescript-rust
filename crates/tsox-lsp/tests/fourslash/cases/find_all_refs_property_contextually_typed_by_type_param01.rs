use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
