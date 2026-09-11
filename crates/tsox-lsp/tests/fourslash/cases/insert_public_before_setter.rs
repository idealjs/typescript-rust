use tsox_lsp::fourslash::{self, Session};


#[test]
fn insert_public_before_setter() {
    let content = r#"class C {
    /**/set Bar(bar:string) {}
}
var o2 = { set Foo(val:number) { } };"#;
    let mut s = Session::new_for_test("insertPublicBeforeSetter", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "public ");
}
